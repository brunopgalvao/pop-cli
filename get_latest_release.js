import fetch from 'node-fetch';
import { Octokit } from '@octokit/rest';

async function getLatestRelease() {
  try {
    const octokit = new Octokit({ request: { fetch } });
    const { data: releases } = await octokit.repos.listReleases({
      owner: "r0gue-io",
      repo: "pop-cli",
    });
    const latestRelease = releases[0];
    const asset = latestRelease.assets.find(asset => asset.name.endsWith("linux-gnu.tar.gz"));
    console.log(`::set-output name=asset_url::${asset.browser_download_url}`);
  } catch (error) {
    console.error(error);
    process.exit(1);
  }
}

getLatestRelease();
