use std::io::{Read, Write};

fn main() {
    let mut bytes = Vec::new();
    let response = match std::io::stdin()
        .take((rcount_package_workbench::MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
    {
        Err(_) => rcount_package_workbench::refusal("Could not read request"),
        Ok(_) => match std::str::from_utf8(&bytes) {
            Ok(input) => rcount_package_workbench::evaluate(input),
            Err(_) => rcount_package_workbench::refusal("Request is not UTF-8"),
        },
    };
    std::io::stdout().write_all(response.as_bytes()).unwrap();
}
