import { describe, it, expect } from 'vitest';
import { BASE_PATH, assetPath, GITHUB_URL } from '../site';

describe('site config', () => {
  it('builds asset paths with the base path', () => {
    expect(assetPath('/svg/logo.svg')).toBe(`${BASE_PATH}/svg/logo.svg`);
  });

  it('exposes the GitHub URL', () => {
    expect(GITHUB_URL).toBe('https://github.com/hieudoanm/grobid.rs');
  });
});
