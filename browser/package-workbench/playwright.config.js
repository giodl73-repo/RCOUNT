import {defineConfig} from '@playwright/test';
export default defineConfig({testDir:'tests',testMatch:'*.spec.js',workers:1,timeout:30000,
  use:{baseURL:process.env.RCOUNT_URL??'http://127.0.0.1:8775/RCOUNT/',headless:true,trace:'retain-on-failure',screenshot:'only-on-failure'},
  webServer:process.env.RCOUNT_URL?undefined:{command:'python -m http.server 8775 --directory ../../target/package-prototype --bind 127.0.0.1',url:'http://127.0.0.1:8775/RCOUNT/',reuseExistingServer:false},
});
