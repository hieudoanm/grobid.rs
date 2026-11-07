export const BASE_PATH =
  process.env.NODE_ENV === 'production' ? '/grobid.rs' : '';

export const assetPath = (path: string) => `${BASE_PATH}${path}`;

export const GITHUB_URL = 'https://github.com/hieudoanm/grobid.rs';
