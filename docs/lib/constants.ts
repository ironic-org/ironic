export const CURRENT_VERSION = '1.1.0';
export const CURRENT_VERSION_TAG = `v${CURRENT_VERSION}`;
export const LATEST_VERSION_LABEL = `Latest: ${CURRENT_VERSION_TAG}`;

export const GIT_BRANCH = import.meta.env.VITE_GIT_BRANCH || 'local';

export const GITHUB_OWNER = 'ironic-org';
export const GITHUB_REPO = 'ironic';
export const GITHUB_URL = `https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}`;
export const GITHUB_API_URL = `https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}`;
