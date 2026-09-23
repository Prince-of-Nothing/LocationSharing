import { createHash, randomBytes, timingSafeEqual } from 'node:crypto';
import * as argon2 from 'argon2';
import { SignJWT, jwtVerify, type JWTPayload } from 'jose';

/**
 * SC-01 Password hashing — Argon2id (OWASP first recommendation).
 * Parameters follow the OWASP Password Storage Cheat Sheet baseline:
 * m=19 MiB, t=2, p=1 (meets the ≥64 MiB / t=3 / p=1 alternative equivalently).
 */
const ARGON2_OPTIONS: argon2.Options = {
  type: argon2.argon2id,
  memoryCost: 19456, // KiB (~19 MiB)
  timeCost: 2,
  parallelism: 1,
};

export async function hashPassword(password: string): Promise<string> {
  return argon2.hash(password, ARGON2_OPTIONS);
}

export async function verifyPassword(hash: string, password: string): Promise<boolean> {
  try {
    return await argon2.verify(hash, password);
  } catch {
    return false;
  }
}

/**
 * SC-02 Weak-password screening (minimum 10 chars per OWASP; reject common
 * breached patterns). Returns list of human-readable issues (empty = ok).
 */
const COMMON_PASSWORDS = new Set([
  'password', 'password1', 'passw0rd', '1234567890', 'qwertyuiop',
  'letmein1', 'iloveyou', 'admin123', 'welcome1', 'abc12345',
]);

export function checkPasswordStrength(pw: string): string[] {
  const issues: string[] = [];
  if (pw.length < 10) issues.push('Must be at least 10 characters long.');
  if (pw.length > 128) issues.push('Must be at most 128 characters long.');
  if (!/[a-z]/.test(pw)) issues.push('Must contain a lowercase letter.');
  if (!/[A-Z]/.test(pw)) issues.push('Must contain an uppercase letter.');
  if (!/[0-9]/.test(pw)) issues.push('Must contain a digit.');
  if (COMMON_PASSWORDS.has(pw.toLowerCase())) issues.push('This password is too common.');
  return issues;
}

export function normalizeEmail(email: string): string {
  return email.trim().toLowerCase();
}

/** SHA-256 hex digest — used to store refresh-token hashes, never raw tokens. */
export function sha256(value: string): string {
  return createHash('sha256').update(value).digest('hex');
}

/** Constant-time comparison helper. */
export function safeEqual(a: string, b: string): boolean {
  const ba = Buffer.from(a);
  const bb = Buffer.from(b);
  if (ba.length !== bb.length) return false;
  return timingSafeEqual(ba, bb);
}

// ---------------------------------------------------------------------------
// SC-03 Secure JWT handling — short-lived HS256 access tokens with jti,
// strict algorithm pinning, and typed `type` claim so refresh material can
// never be replayed as an access token.
// ---------------------------------------------------------------------------

export interface AccessTokenClaims extends JWTPayload {
  sub: string;
  role: 'resident' | 'administrator';
  type: 'access';
  jti: string;
}

function secretKey(secret: string): Uint8Array {
  return new TextEncoder().encode(secret);
}

export async function signAccessToken(
  secret: string,
  claims: { sub: string; role: 'resident' | 'administrator' },
  expiresMinutes: number,
): Promise<string> {
  return await new SignJWT({ ...claims, type: 'access' })
    .setProtectedHeader({ alg: 'HS256', typ: 'JWT' })
    .setJti(randomBytes(16).toString('hex'))
    .setIssuedAt()
    .setIssuer('always-together-api')
    .setExpirationTime(`${expiresMinutes}m`)
    .sign(secretKey(secret));
}

export async function verifyAccessToken(
  secret: string,
  token: string,
): Promise<AccessTokenClaims | null> {
  try {
    const { payload } = await jwtVerify(token, secretKey(secret), {
      algorithms: ['HS256'], // pinned — defends against alg-confusion / "none"
      issuer: 'always-together-api',
    });
    if (payload.type !== 'access' || typeof payload.sub !== 'string') return null;
    return payload as AccessTokenClaims;
  } catch {
    return null; // expired, bad signature, wrong alg — all treated identically
  }
}

/** Opaque, cryptographically-random refresh token (raw value shown once). */
export function generateRefreshToken(): string {
  return randomBytes(32).toString('base64url');
}
