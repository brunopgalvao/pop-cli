async function getLatestRelease() {
  try {
    // Use dynamic import for node-fetch
    const { default: fetch } = await import('node-fetch');
    const { Octokit } = await import('@octokit/rest');

    const octokit = new Octokit({ request: { fetch } });
    const { data: releases } = await octokit.repos.listReleases({
      owner: "r0gue-io",
      repo: "pop-cli",
    });
    const latestRelease = releases[0];
    const asset = latestRelease.assets.find(asset => asset.name.endsWith("linux-gnu.tar.gz"));
    
    // Output the asset URL as a GitHub action output
    console.log(`::set-output name=asset_url::${asset.browser_download_url}`);
  } catch (error) {
    console.error(error);
    process.exit(1);
  }
}

getLatestRelease();
