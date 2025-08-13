const core = require("@actions/core");
const github = require("@actions/github");

async function run() {
  try {
    const context = github.context;

    if (context.eventName === "pull_request") {
      core.setOutput("result", "pr-" + context.issue.number);
    } else if (context.ref === "refs/heads/main") {
      const prNum = await getAssociatedPrNum();
      core.setOutput("result", prNum ? "pr-" + prNum : "main");
    } else {
      core.setOutput("result", "main");
    }
  } catch (error) {
    core.setFailed(error.message);
  }
}

async function getAssociatedPrNum() {
  const prList = await github
    .getOctokit(core.getInput("gh_token"))
    .rest.repos.listPullRequestsAssociatedWithCommit({
      commit_sha: github.context.sha,
      owner: github.context.repo.owner,
      repo: github.context.repo.repo,
    });
  return prList?.data[0]?.number;
}

run();
