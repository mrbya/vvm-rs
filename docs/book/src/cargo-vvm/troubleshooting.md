# Troubleshooting

- Use a fresh output directory for each run.
- If no artifacts are found, confirm the child tests actually captured VVM
  coverage and wrote to the expected directory.
- If a merge fails, check definition fingerprints and schema compatibility.
- If child status is failing, treat the test failure and the reporting result as
  separate questions.
