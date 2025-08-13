module.exports = async ({ github, context }) => {
  if (context.eventName == "pull_request") {
    return "pr-" + context.issue.number;
  }
  if (context.ref == "refs/heads/main") {
    const prNum = (
      await github.rest.repos.listPullRequestsAssociatedWithCommit({
        commit_sha: context.sha,
        owner: context.repo.owner,
        repo: context.repo.repo,
      })
    )?.data[0]?.number;
    return prNum ? "pr-" + prNum : "main";
  }
  return "main";
};
