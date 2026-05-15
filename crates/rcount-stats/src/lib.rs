use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

pub const PPM_DENOMINATOR: u32 = 1_000_000;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RcountStatsError {
    #[error("rational denominator must be positive")]
    NonPositiveDenominator,
    #[error("probability ppm must be at most 1000000: {0}")]
    InvalidProbabilityPpm(u32),
    #[error("risk limit ppm must be between 1 and 999999: {0}")]
    InvalidRiskLimitPpm(u32),
    #[error("division by zero")]
    DivisionByZero,
    #[error("integer overflow in rational arithmetic")]
    RationalOverflow,
    #[error("reported winner and loser votes must both be positive")]
    InvalidBravoReportedVotes,
    #[error("BRAVO replay requires at least one ballot observation")]
    EmptyBravoObservations,
    #[error("reported winner and loser votes must both be positive")]
    InvalidMinervaReportedVotes,
    #[error("Minerva replay requires at least one ballot observation")]
    EmptyMinervaObservations,
    #[error("round-one Minerva replay currently supports only winner/loser observations")]
    InvalidMinervaObservation,
    #[error("bounded martingale upper bound must be positive")]
    InvalidMartingaleUpperBound,
    #[error("bounded martingale null mean must be between zero and the upper bound")]
    InvalidMartingaleNullMean,
    #[error("bounded martingale observation is outside [0, upper_bound]")]
    InvalidMartingaleObservation,
    #[error("bounded martingale update factor is negative")]
    NegativeMartingaleFactor,
    #[error("comparison audit CVR and hand interpretations cannot both be other")]
    InvalidComparisonObservation,
    #[error("reported margin must be positive")]
    InvalidReportedMargin,
    #[error("Kaplan-Markov taint-product replay requires at least one taint")]
    EmptyKaplanMarkovTaints,
    #[error("Kaplan-Markov taint must be less than one")]
    InvalidKaplanMarkovTaint,
    #[error("Kaplan-Markov MACRO replay requires at least one overstatement")]
    EmptyMacroOverstatements,
    #[error("Kaplan-Markov MACRO ballot count must be positive")]
    InvalidMacroBallotCount,
    #[error("Kaplan-Markov MACRO gamma must be greater than one")]
    InvalidMacroGamma,
    #[error("Kaplan-Markov MACRO overstatement must be -2, -1, 0, 1, or 2")]
    InvalidMacroOverstatement,
    #[error("batch comparison totals must be non-negative")]
    NegativeBatchComparisonTotal,
    #[error("integer overflow in batch comparison arithmetic")]
    BatchComparisonOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rational {
    pub numerator: i128,
    pub denominator: i128,
}

impl Rational {
    pub fn new(numerator: i128, denominator: i128) -> Result<Self, RcountStatsError> {
        if denominator <= 0 {
            return Err(RcountStatsError::NonPositiveDenominator);
        }
        let gcd = gcd_i128(numerator, denominator);
        Ok(Self {
            numerator: numerator / gcd,
            denominator: denominator / gcd,
        })
    }

    pub fn zero() -> Self {
        Self {
            numerator: 0,
            denominator: 1,
        }
    }

    pub fn one() -> Self {
        Self {
            numerator: 1,
            denominator: 1,
        }
    }

    pub fn from_ppm(ppm: u32) -> Result<Self, RcountStatsError> {
        validate_probability_ppm(ppm)?;
        Self::new(ppm as i128, PPM_DENOMINATOR as i128)
    }

    pub fn checked_add(self, rhs: Self) -> Result<Self, RcountStatsError> {
        let numerator = self
            .numerator
            .checked_mul(rhs.denominator)
            .and_then(|lhs| {
                rhs.numerator
                    .checked_mul(self.denominator)
                    .and_then(|r| lhs.checked_add(r))
            })
            .ok_or(RcountStatsError::RationalOverflow)?;
        let denominator = self
            .denominator
            .checked_mul(rhs.denominator)
            .ok_or(RcountStatsError::RationalOverflow)?;
        Self::new(numerator, denominator)
    }

    pub fn checked_sub(self, rhs: Self) -> Result<Self, RcountStatsError> {
        let numerator = self
            .numerator
            .checked_mul(rhs.denominator)
            .and_then(|lhs| {
                rhs.numerator
                    .checked_mul(self.denominator)
                    .and_then(|r| lhs.checked_sub(r))
            })
            .ok_or(RcountStatsError::RationalOverflow)?;
        let denominator = self
            .denominator
            .checked_mul(rhs.denominator)
            .ok_or(RcountStatsError::RationalOverflow)?;
        Self::new(numerator, denominator)
    }

    pub fn checked_mul(self, rhs: Self) -> Result<Self, RcountStatsError> {
        let numerator = self
            .numerator
            .checked_mul(rhs.numerator)
            .ok_or(RcountStatsError::RationalOverflow)?;
        let denominator = self
            .denominator
            .checked_mul(rhs.denominator)
            .ok_or(RcountStatsError::RationalOverflow)?;
        Self::new(numerator, denominator)
    }

    pub fn checked_div(self, rhs: Self) -> Result<Self, RcountStatsError> {
        if rhs.numerator == 0 {
            return Err(RcountStatsError::DivisionByZero);
        }
        let numerator = self
            .numerator
            .checked_mul(rhs.denominator)
            .ok_or(RcountStatsError::RationalOverflow)?;
        let denominator = self
            .denominator
            .checked_mul(rhs.numerator)
            .ok_or(RcountStatsError::RationalOverflow)?;
        if denominator < 0 {
            Self::new(-numerator, -denominator)
        } else {
            Self::new(numerator, denominator)
        }
    }

    pub fn ceil_ppm(self) -> Result<u32, RcountStatsError> {
        if self.numerator < 0 {
            return Ok(0);
        }
        let scaled = self
            .numerator
            .checked_mul(PPM_DENOMINATOR as i128)
            .ok_or(RcountStatsError::RationalOverflow)?;
        let ppm = (scaled + self.denominator - 1) / self.denominator;
        Ok(ppm.clamp(0, PPM_DENOMINATOR as i128) as u32)
    }

    pub fn checked_cmp(self, rhs: Self) -> Result<Ordering, RcountStatsError> {
        let lhs = self
            .numerator
            .checked_mul(rhs.denominator)
            .ok_or(RcountStatsError::RationalOverflow)?;
        let rhs = rhs
            .numerator
            .checked_mul(self.denominator)
            .ok_or(RcountStatsError::RationalOverflow)?;
        Ok(lhs.cmp(&rhs))
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        self.checked_cmp(*other)
            .expect("normalized rational comparison overflowed")
    }
}

pub fn validate_probability_ppm(ppm: u32) -> Result<(), RcountStatsError> {
    if ppm > PPM_DENOMINATOR {
        return Err(RcountStatsError::InvalidProbabilityPpm(ppm));
    }
    Ok(())
}

pub fn validate_risk_limit_ppm(ppm: u32) -> Result<(), RcountStatsError> {
    if ppm == 0 || ppm >= PPM_DENOMINATOR {
        return Err(RcountStatsError::InvalidRiskLimitPpm(ppm));
    }
    Ok(())
}

pub fn risk_passes(p_value_ppm: u32, risk_limit_ppm: u32) -> Result<bool, RcountStatsError> {
    validate_probability_ppm(p_value_ppm)?;
    validate_risk_limit_ppm(risk_limit_ppm)?;
    Ok(p_value_ppm <= risk_limit_ppm)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BravoObservation {
    Winner,
    Loser,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BravoStep {
    pub draw_index: u32,
    pub observation: BravoObservation,
    pub likelihood_ratio: Rational,
    pub p_value_ppm: u32,
    pub stop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BravoReplay {
    pub reported_winner_share: Rational,
    pub stop_threshold: Rational,
    pub steps: Vec<BravoStep>,
    pub stopped: bool,
}

pub fn replay_bravo_ballot_polling(
    reported_winner_votes: u64,
    reported_loser_votes: u64,
    risk_limit_ppm: u32,
    observations: &[BravoObservation],
) -> Result<BravoReplay, RcountStatsError> {
    validate_risk_limit_ppm(risk_limit_ppm)?;
    if reported_winner_votes == 0 || reported_loser_votes == 0 {
        return Err(RcountStatsError::InvalidBravoReportedVotes);
    }
    if observations.is_empty() {
        return Err(RcountStatsError::EmptyBravoObservations);
    }

    let total = reported_winner_votes as i128 + reported_loser_votes as i128;
    let winner_share = Rational::new(reported_winner_votes as i128, total)?;
    let loser_share = Rational::new(reported_loser_votes as i128, total)?;
    let two = Rational::new(2, 1)?;
    let winner_factor = two.checked_mul(winner_share)?;
    let loser_factor = two.checked_mul(loser_share)?;
    let risk_limit = Rational::from_ppm(risk_limit_ppm)?;
    let stop_threshold = Rational::one().checked_div(risk_limit)?;

    let mut likelihood_ratio = Rational::one();
    let mut stopped = false;
    let mut steps = Vec::with_capacity(observations.len());
    for (index, observation) in observations.iter().copied().enumerate() {
        let factor = match observation {
            BravoObservation::Winner => winner_factor,
            BravoObservation::Loser => loser_factor,
            BravoObservation::Other => Rational::one(),
        };
        likelihood_ratio = likelihood_ratio.checked_mul(factor)?;
        let p_value = p_value_from_test_statistic(likelihood_ratio)?;
        let stop = likelihood_ratio.checked_cmp(stop_threshold)? != Ordering::Less;
        stopped |= stop;
        steps.push(BravoStep {
            draw_index: index as u32,
            observation,
            likelihood_ratio,
            p_value_ppm: p_value,
            stop,
        });
    }

    Ok(BravoReplay {
        reported_winner_share: winner_share,
        stop_threshold,
        steps,
        stopped,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinervaRoundStep {
    pub round_index: u32,
    pub sample_size: u32,
    pub winner_ballots: u32,
    pub alternative_tail: Rational,
    pub null_tail: Rational,
    pub likelihood_ratio: Rational,
    pub p_value_ppm: u32,
    pub stop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinervaRoundReplay {
    pub reported_winner_share: Rational,
    pub stop_threshold: Rational,
    pub steps: Vec<MinervaRoundStep>,
    pub stopped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinervaRoundObservationSet {
    pub round_index: u32,
    pub observations: Vec<BravoObservation>,
}

pub fn replay_minerva_round_one_ballot_polling(
    reported_winner_votes: u64,
    reported_loser_votes: u64,
    risk_limit_ppm: u32,
    observations: &[BravoObservation],
) -> Result<MinervaRoundReplay, RcountStatsError> {
    replay_minerva_ballot_polling_rounds(
        reported_winner_votes,
        reported_loser_votes,
        risk_limit_ppm,
        &[MinervaRoundObservationSet {
            round_index: 0,
            observations: observations.to_vec(),
        }],
    )
}

pub fn replay_minerva_ballot_polling_rounds(
    reported_winner_votes: u64,
    reported_loser_votes: u64,
    risk_limit_ppm: u32,
    rounds: &[MinervaRoundObservationSet],
) -> Result<MinervaRoundReplay, RcountStatsError> {
    validate_risk_limit_ppm(risk_limit_ppm)?;
    if reported_winner_votes == 0 || reported_loser_votes == 0 {
        return Err(RcountStatsError::InvalidMinervaReportedVotes);
    }
    if rounds.is_empty() {
        return Err(RcountStatsError::EmptyMinervaObservations);
    }

    let total = reported_winner_votes as i128 + reported_loser_votes as i128;
    let winner_share = Rational::new(reported_winner_votes as i128, total)?;
    let risk_limit = Rational::from_ppm(risk_limit_ppm)?;
    let stop_threshold = Rational::one().checked_div(risk_limit)?;

    let mut cumulative = Vec::new();
    let mut steps = Vec::with_capacity(rounds.len());
    let mut stopped = false;
    for round in rounds {
        if round.observations.is_empty() {
            return Err(RcountStatsError::EmptyMinervaObservations);
        }
        cumulative.extend(round.observations.iter().copied());
        let step =
            minerva_round_step(round.round_index, winner_share, stop_threshold, &cumulative)?;
        stopped |= step.stop;
        steps.push(step);
    }

    Ok(MinervaRoundReplay {
        reported_winner_share: winner_share,
        stop_threshold,
        steps,
        stopped,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedMeanMartingaleConfig {
    pub null_mean: Rational,
    pub upper_bound: Rational,
    pub risk_limit_ppm: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedMeanMartingaleObservation {
    pub value: Rational,
    pub bet: Rational,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedMeanMartingaleStep {
    pub step_index: u32,
    pub value: Rational,
    pub bet: Rational,
    pub update_factor: Rational,
    pub martingale: Rational,
    pub p_value_ppm: u32,
    pub stop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedMeanMartingaleReplay {
    pub stop_threshold: Rational,
    pub steps: Vec<BoundedMeanMartingaleStep>,
    pub stopped: bool,
}

pub fn replay_fixed_bet_bounded_mean_martingale(
    config: BoundedMeanMartingaleConfig,
    observations: &[BoundedMeanMartingaleObservation],
) -> Result<BoundedMeanMartingaleReplay, RcountStatsError> {
    validate_risk_limit_ppm(config.risk_limit_ppm)?;
    if config.upper_bound.checked_cmp(Rational::zero())? != Ordering::Greater {
        return Err(RcountStatsError::InvalidMartingaleUpperBound);
    }
    if config.null_mean.checked_cmp(Rational::zero())? == Ordering::Less
        || config.null_mean.checked_cmp(config.upper_bound)? == Ordering::Greater
    {
        return Err(RcountStatsError::InvalidMartingaleNullMean);
    }

    let risk_limit = Rational::from_ppm(config.risk_limit_ppm)?;
    let stop_threshold = Rational::one().checked_div(risk_limit)?;
    let mut martingale = Rational::one();
    let mut stopped = false;
    let mut steps = Vec::with_capacity(observations.len());

    for (index, observation) in observations.iter().copied().enumerate() {
        if observation.value.checked_cmp(Rational::zero())? == Ordering::Less
            || observation.value.checked_cmp(config.upper_bound)? == Ordering::Greater
        {
            return Err(RcountStatsError::InvalidMartingaleObservation);
        }

        let centered = observation.value.checked_sub(config.null_mean)?;
        let scaled = observation
            .bet
            .checked_mul(centered)?
            .checked_div(config.upper_bound)?;
        let update_factor = Rational::one().checked_add(scaled)?;
        if update_factor.checked_cmp(Rational::zero())? == Ordering::Less {
            return Err(RcountStatsError::NegativeMartingaleFactor);
        }
        martingale = martingale.checked_mul(update_factor)?;
        let p_value = p_value_from_test_statistic(martingale)?;
        let stop = martingale.checked_cmp(stop_threshold)? != Ordering::Less;
        stopped |= stop;
        steps.push(BoundedMeanMartingaleStep {
            step_index: index as u32,
            value: observation.value,
            bet: observation.bet,
            update_factor,
            martingale,
            p_value_ppm: p_value,
            stop,
        });
    }

    Ok(BoundedMeanMartingaleReplay {
        stop_threshold,
        steps,
        stopped,
    })
}

pub fn p_value_from_test_statistic(test_statistic: Rational) -> Result<u32, RcountStatsError> {
    if test_statistic.checked_cmp(Rational::zero())? != Ordering::Greater {
        return Ok(PPM_DENOMINATOR);
    }
    let p_value = Rational::one().checked_div(test_statistic)?;
    p_value.ceil_ppm()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluralityComparisonSelection {
    Winner,
    Loser,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluralityComparisonObservation {
    pub cvr_selection: PluralityComparisonSelection,
    pub hand_selection: PluralityComparisonSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluralityOverstatement {
    pub cvr_margin_contribution: i8,
    pub hand_margin_contribution: i8,
    pub overstatement: i8,
}

pub fn plurality_winner_loser_overstatement(
    observation: PluralityComparisonObservation,
) -> Result<PluralityOverstatement, RcountStatsError> {
    if observation.cvr_selection == PluralityComparisonSelection::Other
        && observation.hand_selection == PluralityComparisonSelection::Other
    {
        return Err(RcountStatsError::InvalidComparisonObservation);
    }
    let cvr_margin_contribution = margin_contribution(observation.cvr_selection);
    let hand_margin_contribution = margin_contribution(observation.hand_selection);
    Ok(PluralityOverstatement {
        cvr_margin_contribution,
        hand_margin_contribution,
        overstatement: cvr_margin_contribution - hand_margin_contribution,
    })
}

pub fn overstatement_taint(
    overstatement: i8,
    reported_margin: i64,
) -> Result<Rational, RcountStatsError> {
    if reported_margin <= 0 {
        return Err(RcountStatsError::InvalidReportedMargin);
    }
    Rational::new(overstatement as i128, reported_margin as i128)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaplanMarkovTaintStep {
    pub step_index: u32,
    pub taint: Rational,
    pub p_value: Rational,
    pub p_value_ppm: u32,
    pub stop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaplanMarkovTaintProductReplay {
    pub risk_limit: Rational,
    pub steps: Vec<KaplanMarkovTaintStep>,
    pub stopped: bool,
}

pub fn replay_kaplan_markov_taint_product(
    risk_limit_ppm: u32,
    taints: &[Rational],
) -> Result<KaplanMarkovTaintProductReplay, RcountStatsError> {
    validate_risk_limit_ppm(risk_limit_ppm)?;
    if taints.is_empty() {
        return Err(RcountStatsError::EmptyKaplanMarkovTaints);
    }

    let risk_limit = Rational::from_ppm(risk_limit_ppm)?;
    let mut p_value = Rational::one();
    let mut stopped = false;
    let mut steps = Vec::with_capacity(taints.len());

    for (index, taint) in taints.iter().copied().enumerate() {
        if taint.checked_cmp(Rational::one())? != Ordering::Less {
            return Err(RcountStatsError::InvalidKaplanMarkovTaint);
        }
        let bounded_taint = if taint.checked_cmp(Rational::zero())? == Ordering::Less {
            Rational::zero()
        } else {
            taint
        };
        p_value = p_value.checked_mul(Rational::one().checked_sub(bounded_taint)?)?;
        let stop = p_value.checked_cmp(risk_limit)? != Ordering::Greater;
        stopped |= stop;
        steps.push(KaplanMarkovTaintStep {
            step_index: index as u32,
            taint,
            p_value,
            p_value_ppm: p_value.ceil_ppm()?,
            stop,
        });
    }

    Ok(KaplanMarkovTaintProductReplay {
        risk_limit,
        steps,
        stopped,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaplanMarkovMacroConfig {
    pub ballot_count: u64,
    pub reported_margin: u64,
    pub gamma: Rational,
    pub risk_limit_ppm: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaplanMarkovMacroStep {
    pub step_index: u32,
    pub overstatement: i8,
    pub p_value: Rational,
    pub p_value_ppm: u32,
    pub stop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaplanMarkovMacroReplay {
    pub upper_bound: Rational,
    pub no_error_factor: Rational,
    pub risk_limit: Rational,
    pub steps: Vec<KaplanMarkovMacroStep>,
    pub stopped: bool,
}

pub fn replay_kaplan_markov_macro_bound(
    config: KaplanMarkovMacroConfig,
    overstatements: &[i8],
) -> Result<KaplanMarkovMacroReplay, RcountStatsError> {
    validate_risk_limit_ppm(config.risk_limit_ppm)?;
    if overstatements.is_empty() {
        return Err(RcountStatsError::EmptyMacroOverstatements);
    }
    if config.ballot_count == 0 || config.reported_margin == 0 {
        return Err(RcountStatsError::InvalidMacroBallotCount);
    }
    if config.gamma.checked_cmp(Rational::one())? != Ordering::Greater {
        return Err(RcountStatsError::InvalidMacroGamma);
    }

    let two = Rational::new(2, 1)?;
    let upper_bound = two.checked_mul(config.gamma)?.checked_mul(Rational::new(
        config.ballot_count as i128,
        config.reported_margin as i128,
    )?)?;
    let no_error_factor = Rational::one().checked_sub(Rational::one().checked_div(upper_bound)?)?;
    let one_vote_error_factor = no_error_factor.checked_div(
        Rational::one()
            .checked_sub(Rational::one().checked_div(two.checked_mul(config.gamma)?)?)?,
    )?;
    let two_vote_error_factor = no_error_factor
        .checked_div(Rational::one().checked_sub(Rational::one().checked_div(config.gamma)?)?)?;
    let risk_limit = Rational::from_ppm(config.risk_limit_ppm)?;

    let mut p_value = Rational::one();
    let mut stopped = false;
    let mut steps = Vec::with_capacity(overstatements.len());
    for (index, overstatement) in overstatements.iter().copied().enumerate() {
        let factor = match overstatement {
            -2..=0 => no_error_factor,
            1 => one_vote_error_factor,
            2 => two_vote_error_factor,
            _ => return Err(RcountStatsError::InvalidMacroOverstatement),
        };
        p_value = p_value.checked_mul(factor)?;
        let stop = p_value.checked_cmp(risk_limit)? != Ordering::Greater;
        stopped |= stop;
        steps.push(KaplanMarkovMacroStep {
            step_index: index as u32,
            overstatement,
            p_value,
            p_value_ppm: p_value.ceil_ppm()?,
            stop,
        });
    }

    Ok(KaplanMarkovMacroReplay {
        upper_bound,
        no_error_factor,
        risk_limit,
        steps,
        stopped,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchPluralityTotals {
    pub winner_votes: i64,
    pub loser_votes: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchPluralityComparison {
    pub reported: BatchPluralityTotals,
    pub hand: BatchPluralityTotals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchPluralityOverstatement {
    pub reported_margin: i64,
    pub hand_margin: i64,
    pub overstatement: i64,
}

pub fn batch_plurality_overstatement(
    comparison: BatchPluralityComparison,
) -> Result<BatchPluralityOverstatement, RcountStatsError> {
    validate_batch_totals(comparison.reported)?;
    validate_batch_totals(comparison.hand)?;
    let reported_margin = margin_from_totals(comparison.reported)?;
    let hand_margin = margin_from_totals(comparison.hand)?;
    let overstatement = reported_margin
        .checked_sub(hand_margin)
        .ok_or(RcountStatsError::BatchComparisonOverflow)?;
    Ok(BatchPluralityOverstatement {
        reported_margin,
        hand_margin,
        overstatement,
    })
}

fn validate_batch_totals(totals: BatchPluralityTotals) -> Result<(), RcountStatsError> {
    if totals.winner_votes < 0 || totals.loser_votes < 0 {
        return Err(RcountStatsError::NegativeBatchComparisonTotal);
    }
    Ok(())
}

fn margin_from_totals(totals: BatchPluralityTotals) -> Result<i64, RcountStatsError> {
    totals
        .winner_votes
        .checked_sub(totals.loser_votes)
        .ok_or(RcountStatsError::BatchComparisonOverflow)
}

fn margin_contribution(selection: PluralityComparisonSelection) -> i8 {
    match selection {
        PluralityComparisonSelection::Winner => 1,
        PluralityComparisonSelection::Loser => -1,
        PluralityComparisonSelection::Other => 0,
    }
}

fn binomial_tail(
    sample_size: u32,
    threshold: u32,
    success_probability: Rational,
) -> Result<Rational, RcountStatsError> {
    let failure_probability = Rational::one().checked_sub(success_probability)?;
    let mut tail = Rational::zero();
    for successes in threshold..=sample_size {
        let coefficient = Rational::new(
            i128::try_from(binomial_coefficient(sample_size, successes)?)
                .map_err(|_| RcountStatsError::RationalOverflow)?,
            1,
        )?;
        let success_term = rational_pow(success_probability, successes)?;
        let failure_term = rational_pow(failure_probability, sample_size - successes)?;
        tail = tail.checked_add(
            coefficient
                .checked_mul(success_term)?
                .checked_mul(failure_term)?,
        )?;
    }
    Ok(tail)
}

fn minerva_round_step(
    round_index: u32,
    winner_share: Rational,
    stop_threshold: Rational,
    observations: &[BravoObservation],
) -> Result<MinervaRoundStep, RcountStatsError> {
    let mut winner_ballots = 0_u32;
    for observation in observations {
        match observation {
            BravoObservation::Winner => winner_ballots += 1,
            BravoObservation::Loser => {}
            BravoObservation::Other => return Err(RcountStatsError::InvalidMinervaObservation),
        }
    }

    let sample_size =
        u32::try_from(observations.len()).map_err(|_| RcountStatsError::RationalOverflow)?;
    let null_share = Rational::new(1, 2)?;
    let alternative_tail = binomial_tail(sample_size, winner_ballots, winner_share)?;
    let null_tail = binomial_tail(sample_size, winner_ballots, null_share)?;
    let likelihood_ratio = alternative_tail.checked_div(null_tail)?;
    let p_value_ppm = p_value_from_test_statistic(likelihood_ratio)?;
    let stop = likelihood_ratio.checked_cmp(stop_threshold)? != Ordering::Less;

    Ok(MinervaRoundStep {
        round_index,
        sample_size,
        winner_ballots,
        alternative_tail,
        null_tail,
        likelihood_ratio,
        p_value_ppm,
        stop,
    })
}

fn binomial_coefficient(n: u32, k: u32) -> Result<u128, RcountStatsError> {
    let k = k.min(n - k);
    let mut coefficient = 1_u128;
    for i in 1..=k {
        coefficient = coefficient
            .checked_mul((n - k + i) as u128)
            .ok_or(RcountStatsError::RationalOverflow)?
            / i as u128;
    }
    Ok(coefficient)
}

fn rational_pow(base: Rational, exponent: u32) -> Result<Rational, RcountStatsError> {
    let mut value = Rational::one();
    for _ in 0..exponent {
        value = value.checked_mul(base)?;
    }
    Ok(value)
}

fn gcd_i128(mut a: i128, mut b: i128) -> i128 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    if a == 0 {
        1
    } else {
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rational_normalizes_sign_and_gcd() {
        assert_eq!(
            Rational::new(10, 20).unwrap(),
            Rational {
                numerator: 1,
                denominator: 2
            }
        );
        assert_eq!(
            Rational::new(-6, 9).unwrap(),
            Rational {
                numerator: -2,
                denominator: 3
            }
        );
    }

    #[test]
    fn rational_arithmetic_is_exact() {
        let one_half = Rational::new(1, 2).unwrap();
        let one_third = Rational::new(1, 3).unwrap();
        assert_eq!(
            one_half.checked_add(one_third).unwrap(),
            Rational::new(5, 6).unwrap()
        );
        assert_eq!(
            one_half.checked_sub(one_third).unwrap(),
            Rational::new(1, 6).unwrap()
        );
        assert_eq!(
            one_half.checked_mul(one_third).unwrap(),
            Rational::new(1, 6).unwrap()
        );
        assert_eq!(
            one_half.checked_div(one_third).unwrap(),
            Rational::new(3, 2).unwrap()
        );
        assert_eq!(one_half.checked_cmp(one_third).unwrap(), Ordering::Greater);
    }

    #[test]
    fn ppm_helpers_validate_probability_and_risk_limit() {
        assert_eq!(
            Rational::from_ppm(125_000).unwrap(),
            Rational::new(1, 8).unwrap()
        );
        assert_eq!(Rational::new(1, 3).unwrap().ceil_ppm().unwrap(), 333_334);
        assert!(validate_probability_ppm(1_000_000).is_ok());
        assert!(validate_risk_limit_ppm(999_999).is_ok());
        assert!(validate_risk_limit_ppm(1_000_000).is_err());
        assert_eq!(risk_passes(50_000, 100_000).unwrap(), true);
        assert_eq!(risk_passes(150_000, 100_000).unwrap(), false);
    }

    #[test]
    fn bravo_replay_stops_when_likelihood_ratio_crosses_threshold() {
        let replay = replay_bravo_ballot_polling(
            3,
            1,
            100_000,
            &[
                BravoObservation::Winner,
                BravoObservation::Winner,
                BravoObservation::Winner,
                BravoObservation::Winner,
                BravoObservation::Winner,
                BravoObservation::Winner,
            ],
        )
        .unwrap();

        assert_eq!(replay.reported_winner_share, Rational::new(3, 4).unwrap());
        assert_eq!(replay.stop_threshold, Rational::new(10, 1).unwrap());
        assert_eq!(
            replay.steps.last().unwrap().likelihood_ratio,
            Rational::new(729, 64).unwrap()
        );
        assert_eq!(replay.steps.last().unwrap().p_value_ppm, 87_792);
        assert!(replay.stopped);
    }

    #[test]
    fn bravo_replay_treats_other_observations_as_neutral() {
        let replay = replay_bravo_ballot_polling(
            3,
            1,
            100_000,
            &[
                BravoObservation::Winner,
                BravoObservation::Other,
                BravoObservation::Loser,
            ],
        )
        .unwrap();

        assert_eq!(
            replay.steps.last().unwrap().likelihood_ratio,
            Rational::new(3, 4).unwrap()
        );
        assert!(!replay.stopped);
    }

    #[test]
    fn minerva_round_one_replay_stops_on_all_winner_sample() {
        let replay = replay_minerva_round_one_ballot_polling(
            3,
            1,
            100_000,
            &[
                BravoObservation::Winner,
                BravoObservation::Winner,
                BravoObservation::Winner,
                BravoObservation::Winner,
                BravoObservation::Winner,
                BravoObservation::Winner,
            ],
        )
        .unwrap();

        let step = replay.steps.last().unwrap();
        assert_eq!(replay.reported_winner_share, Rational::new(3, 4).unwrap());
        assert_eq!(step.alternative_tail, Rational::new(729, 4096).unwrap());
        assert_eq!(step.null_tail, Rational::new(1, 64).unwrap());
        assert_eq!(step.likelihood_ratio, Rational::new(729, 64).unwrap());
        assert_eq!(step.p_value_ppm, 87_792);
        assert!(step.stop);
        assert!(replay.stopped);
    }

    #[test]
    fn minerva_round_one_replay_continues_below_threshold() {
        let replay = replay_minerva_round_one_ballot_polling(
            3,
            1,
            100_000,
            &[
                BravoObservation::Winner,
                BravoObservation::Winner,
                BravoObservation::Winner,
                BravoObservation::Winner,
                BravoObservation::Winner,
            ],
        )
        .unwrap();

        let step = replay.steps.last().unwrap();
        assert_eq!(step.likelihood_ratio, Rational::new(243, 32).unwrap());
        assert_eq!(step.p_value_ppm, 131_688);
        assert!(!step.stop);
        assert!(!replay.stopped);
    }

    #[test]
    fn minerva_round_one_rejects_other_observations() {
        let err = replay_minerva_round_one_ballot_polling(
            3,
            1,
            100_000,
            &[BravoObservation::Winner, BravoObservation::Other],
        )
        .expect_err("other observations are outside the round-one Minerva surface");

        assert_eq!(err, RcountStatsError::InvalidMinervaObservation);
    }

    #[test]
    fn minerva_multi_round_replay_uses_cumulative_observations() {
        let replay = replay_minerva_ballot_polling_rounds(
            3,
            1,
            100_000,
            &[
                MinervaRoundObservationSet {
                    round_index: 0,
                    observations: vec![
                        BravoObservation::Winner,
                        BravoObservation::Winner,
                        BravoObservation::Winner,
                        BravoObservation::Winner,
                        BravoObservation::Winner,
                    ],
                },
                MinervaRoundObservationSet {
                    round_index: 1,
                    observations: vec![BravoObservation::Winner],
                },
            ],
        )
        .unwrap();

        assert_eq!(replay.steps.len(), 2);
        assert_eq!(
            replay.steps[0].likelihood_ratio,
            Rational::new(243, 32).unwrap()
        );
        assert_eq!(replay.steps[0].p_value_ppm, 131_688);
        assert!(!replay.steps[0].stop);
        assert_eq!(
            replay.steps[1].likelihood_ratio,
            Rational::new(729, 64).unwrap()
        );
        assert_eq!(replay.steps[1].p_value_ppm, 87_792);
        assert!(replay.steps[1].stop);
        assert!(replay.stopped);
    }

    #[test]
    fn fixed_bet_bounded_mean_martingale_replays_exactly() {
        let replay = replay_fixed_bet_bounded_mean_martingale(
            BoundedMeanMartingaleConfig {
                null_mean: Rational::new(1, 2).unwrap(),
                upper_bound: Rational::one(),
                risk_limit_ppm: 250_000,
            },
            &[
                BoundedMeanMartingaleObservation {
                    value: Rational::one(),
                    bet: Rational::one(),
                },
                BoundedMeanMartingaleObservation {
                    value: Rational::one(),
                    bet: Rational::one(),
                },
                BoundedMeanMartingaleObservation {
                    value: Rational::one(),
                    bet: Rational::one(),
                },
                BoundedMeanMartingaleObservation {
                    value: Rational::one(),
                    bet: Rational::one(),
                },
            ],
        )
        .unwrap();

        assert_eq!(replay.stop_threshold, Rational::new(4, 1).unwrap());
        assert_eq!(
            replay.steps.last().unwrap().martingale,
            Rational::new(81, 16).unwrap()
        );
        assert_eq!(replay.steps.last().unwrap().p_value_ppm, 197_531);
        assert!(replay.stopped);
    }

    #[test]
    fn fixed_bet_bounded_mean_martingale_rejects_out_of_range_values() {
        let err = replay_fixed_bet_bounded_mean_martingale(
            BoundedMeanMartingaleConfig {
                null_mean: Rational::new(1, 2).unwrap(),
                upper_bound: Rational::one(),
                risk_limit_ppm: 250_000,
            },
            &[BoundedMeanMartingaleObservation {
                value: Rational::new(2, 1).unwrap(),
                bet: Rational::one(),
            }],
        )
        .unwrap_err();

        assert_eq!(err, RcountStatsError::InvalidMartingaleObservation);
    }

    #[test]
    fn plurality_comparison_overstatement_values_are_exact() {
        assert_eq!(
            plurality_winner_loser_overstatement(PluralityComparisonObservation {
                cvr_selection: PluralityComparisonSelection::Winner,
                hand_selection: PluralityComparisonSelection::Winner,
            })
            .unwrap()
            .overstatement,
            0
        );
        assert_eq!(
            plurality_winner_loser_overstatement(PluralityComparisonObservation {
                cvr_selection: PluralityComparisonSelection::Winner,
                hand_selection: PluralityComparisonSelection::Other,
            })
            .unwrap()
            .overstatement,
            1
        );
        assert_eq!(
            plurality_winner_loser_overstatement(PluralityComparisonObservation {
                cvr_selection: PluralityComparisonSelection::Winner,
                hand_selection: PluralityComparisonSelection::Loser,
            })
            .unwrap()
            .overstatement,
            2
        );
        assert_eq!(
            plurality_winner_loser_overstatement(PluralityComparisonObservation {
                cvr_selection: PluralityComparisonSelection::Loser,
                hand_selection: PluralityComparisonSelection::Winner,
            })
            .unwrap()
            .overstatement,
            -2
        );
    }

    #[test]
    fn plurality_comparison_rejects_double_other_observation() {
        let err = plurality_winner_loser_overstatement(PluralityComparisonObservation {
            cvr_selection: PluralityComparisonSelection::Other,
            hand_selection: PluralityComparisonSelection::Other,
        })
        .unwrap_err();

        assert_eq!(err, RcountStatsError::InvalidComparisonObservation);
    }

    #[test]
    fn overstatement_taint_normalizes_by_reported_margin() {
        assert_eq!(
            overstatement_taint(2, 40).unwrap(),
            Rational::new(1, 20).unwrap()
        );
        assert_eq!(
            overstatement_taint(-1, 40).unwrap(),
            Rational::new(-1, 40).unwrap()
        );
    }

    #[test]
    fn overstatement_taint_rejects_nonpositive_margin() {
        assert_eq!(
            overstatement_taint(1, 0).unwrap_err(),
            RcountStatsError::InvalidReportedMargin
        );
    }

    #[test]
    fn kaplan_markov_taint_product_accumulates_running_p_value() {
        let replay = replay_kaplan_markov_taint_product(
            300_000,
            &[Rational::new(1, 2).unwrap(), Rational::new(1, 2).unwrap()],
        )
        .unwrap();

        assert_eq!(replay.risk_limit, Rational::new(3, 10).unwrap());
        assert_eq!(replay.steps[0].p_value, Rational::new(1, 2).unwrap());
        assert_eq!(replay.steps[0].p_value_ppm, 500_000);
        assert!(!replay.steps[0].stop);
        assert_eq!(replay.steps[1].p_value, Rational::new(1, 4).unwrap());
        assert_eq!(replay.steps[1].p_value_ppm, 250_000);
        assert!(replay.steps[1].stop);
        assert!(replay.stopped);
    }

    #[test]
    fn kaplan_markov_taint_product_is_conservative_for_negative_taints() {
        let replay = replay_kaplan_markov_taint_product(
            100_000,
            &[Rational::new(-1, 20).unwrap(), Rational::new(0, 1).unwrap()],
        )
        .unwrap();

        assert_eq!(replay.steps[0].p_value, Rational::one());
        assert_eq!(replay.steps[1].p_value, Rational::one());
        assert!(!replay.stopped);
    }

    #[test]
    fn kaplan_markov_taint_product_rejects_empty_and_unit_taints() {
        assert_eq!(
            replay_kaplan_markov_taint_product(100_000, &[]).unwrap_err(),
            RcountStatsError::EmptyKaplanMarkovTaints
        );
        assert_eq!(
            replay_kaplan_markov_taint_product(100_000, &[Rational::one()]).unwrap_err(),
            RcountStatsError::InvalidKaplanMarkovTaint
        );
    }

    #[test]
    fn kaplan_markov_macro_bound_matches_published_product_shape() {
        let replay = replay_kaplan_markov_macro_bound(
            KaplanMarkovMacroConfig {
                ballot_count: 100,
                reported_margin: 10,
                gamma: Rational::new(11, 10).unwrap(),
                risk_limit_ppm: 100_000,
            },
            &[0, 1, 2],
        )
        .unwrap();

        assert_eq!(replay.upper_bound, Rational::new(22, 1).unwrap());
        assert_eq!(replay.no_error_factor, Rational::new(21, 22).unwrap());
        assert_eq!(replay.steps[0].p_value, Rational::new(21, 22).unwrap());
        assert_eq!(replay.steps[1].p_value, Rational::new(147, 88).unwrap());
        assert_eq!(replay.steps[2].p_value, Rational::new(3087, 176).unwrap());
        assert_eq!(replay.steps[2].p_value_ppm, 1_000_000);
        assert!(!replay.stopped);
    }

    #[test]
    fn kaplan_markov_macro_bound_stops_on_no_error_sample() {
        let replay = replay_kaplan_markov_macro_bound(
            KaplanMarkovMacroConfig {
                ballot_count: 100,
                reported_margin: 10,
                gamma: Rational::new(11, 10).unwrap(),
                risk_limit_ppm: 500_000,
            },
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        )
        .unwrap();

        assert_eq!(replay.steps[1].p_value, Rational::new(441, 484).unwrap());
        assert!(replay.steps.last().unwrap().stop);
        assert!(replay.stopped);
    }

    #[test]
    fn kaplan_markov_macro_bound_rejects_missing_design_inputs() {
        let config = KaplanMarkovMacroConfig {
            ballot_count: 100,
            reported_margin: 10,
            gamma: Rational::new(11, 10).unwrap(),
            risk_limit_ppm: 100_000,
        };
        assert_eq!(
            replay_kaplan_markov_macro_bound(config, &[]).unwrap_err(),
            RcountStatsError::EmptyMacroOverstatements
        );
        assert_eq!(
            replay_kaplan_markov_macro_bound(
                KaplanMarkovMacroConfig {
                    ballot_count: 0,
                    ..config
                },
                &[0],
            )
            .unwrap_err(),
            RcountStatsError::InvalidMacroBallotCount
        );
        assert_eq!(
            replay_kaplan_markov_macro_bound(
                KaplanMarkovMacroConfig {
                    gamma: Rational::one(),
                    ..config
                },
                &[0],
            )
            .unwrap_err(),
            RcountStatsError::InvalidMacroGamma
        );
        assert_eq!(
            replay_kaplan_markov_macro_bound(config, &[3]).unwrap_err(),
            RcountStatsError::InvalidMacroOverstatement
        );
    }

    #[test]
    fn batch_plurality_overstatement_compares_reported_and_hand_margins() {
        let overstatement = batch_plurality_overstatement(BatchPluralityComparison {
            reported: BatchPluralityTotals {
                winner_votes: 40,
                loser_votes: 20,
            },
            hand: BatchPluralityTotals {
                winner_votes: 38,
                loser_votes: 22,
            },
        })
        .unwrap();

        assert_eq!(
            overstatement,
            BatchPluralityOverstatement {
                reported_margin: 20,
                hand_margin: 16,
                overstatement: 4,
            }
        );
        assert_eq!(
            Rational::new(
                overstatement.overstatement as i128,
                overstatement.reported_margin as i128
            )
            .unwrap(),
            Rational::new(1, 5).unwrap()
        );
    }

    #[test]
    fn batch_plurality_overstatement_can_be_negative() {
        let overstatement = batch_plurality_overstatement(BatchPluralityComparison {
            reported: BatchPluralityTotals {
                winner_votes: 40,
                loser_votes: 20,
            },
            hand: BatchPluralityTotals {
                winner_votes: 41,
                loser_votes: 19,
            },
        })
        .unwrap();

        assert_eq!(overstatement.overstatement, -2);
    }

    #[test]
    fn batch_plurality_overstatement_rejects_negative_totals() {
        let err = batch_plurality_overstatement(BatchPluralityComparison {
            reported: BatchPluralityTotals {
                winner_votes: -1,
                loser_votes: 20,
            },
            hand: BatchPluralityTotals {
                winner_votes: 40,
                loser_votes: 20,
            },
        })
        .unwrap_err();

        assert_eq!(err, RcountStatsError::NegativeBatchComparisonTotal);
    }
}
