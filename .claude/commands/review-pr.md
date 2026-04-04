Review the current pull request for code quality, security, and correctness.

Steps:
1. Get the PR diff: run `gh pr diff`
2. Get PR metadata: run `gh pr view`
3. Review for:
   - Logic errors or edge cases
   - Security issues (injection, auth, input validation)
   - Missing tests
   - Code style inconsistencies
   - Documentation gaps
4. Summarize findings as: **Must fix**, **Should fix**, **Nice to have**

$ARGUMENTS
