import { Test } from '@nestjs/testing';
import { INestApplication, ValidationPipe } from '@nestjs/common';
import request from 'supertest';
import { AppModule } from '../src/app.module';
import { DATA_STORE, InMemoryStore } from '../src/db/in-memory.repository';

/**
 * Security test-suite for the MVP controls SC-01..SC-05, SC-07.
 * Runs fully in-memory (no Docker required); the same suite is executed
 * against the containerized stack in CI (see .github/workflows/ci.yml).
 */
describe('Security controls (e2e)', () => {
  let app: INestApplication;
  let store: InMemoryStore;

  const strongPw = 'Correct-Horse-9';

  beforeAll(async () => {
    process.env.JWT_SECRET = 'test-secret-key-at-least-32-characters-long!!';
    process.env.ACCESS_TOKEN_EXPIRE_MINUTES = '15';
    process.env.RATE_LIMIT_MAX = '1000'; // keep limiter out of the way except its own test
    process.env.LOGIN_MAX_ATTEMPTS = '5';

    const moduleRef = await Test.createTestingModule({ imports: [AppModule] }).compile();
    app = moduleRef.createNestApplication();
    app.useGlobalPipes(
      new ValidationPipe({ whitelist: true, forbidNonWhitelisted: true, transform: true }),
    );
    app.setGlobalPrefix('/api/v1', { exclude: ['health/live', 'health/ready'] });
    store = moduleRef.get<InMemoryStore>(DATA_STORE);
    await app.init();
  });

  afterAll(async () => {
    await app.close();
  });

  const api = () => request(app.getHttpServer());
  const url = (p: string) => `/api/v1${p}`;

  async function registerAndLogin(email: string, password = strongPw) {
    await api().post(url('/auth/register')).send({ email, password, fullName: `User ${email}` }).expect(201);
    const res = await api().post(url('/auth/login')).send({ email, password }).expect(200);
    return res.body as { accessToken: string; refreshToken: string };
  }

  describe('SC-01 password hashing', () => {
    it('stores Argon2id hashes, never plaintext', async () => {
      await api().post(url('/auth/register')).send({
        email: 'hash@example.com', password: strongPw, fullName: 'Hash Test',
      }).expect(201);
      const u = store.users.findByEmailNormalized('hash@example.com')!;
      expect(u.passwordHash).toContain('$argon2id$');
      expect(u.passwordHash).not.toContain(strongPw);
    });
  });

  describe('SC-02 password policy & enumeration defense', () => {
    it('rejects weak passwords with field-level detail', async () => {
      const res = await api().post(url('/auth/register')).send({
        email: 'weak@example.com', password: 'short1A', fullName: 'Weak',
      }).expect(400);
      expect(res.body.code).toBe('WEAK_PASSWORD');
      expect(res.body.fields.password.length).toBeGreaterThan(0);
    });

    it('duplicate registration returns a generic error (no enumeration)', async () => {
      await api().post(url('/auth/register')).send({ email: 'dup@example.com', password: strongPw, fullName: 'A' }).expect(201);
      const res = await api().post(url('/auth/register')).send({ email: 'dup@example.com', password: strongPw, fullName: 'B' }).expect(400);
      expect(res.body.code).toBe('REGISTRATION_FAILED');
      expect(JSON.stringify(res.body)).not.toMatch(/already exists|taken/i);
    });
  });

  describe('SC-03 JWT handling', () => {
    it('rejects unsigned/forged tokens', async () => {
      const forged =
        'eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.' +
        Buffer.from(JSON.stringify({ sub: 'someone', type: 'access', role: 'administrator' })).toString('base64url') +
        '.';
      await api().get(url('/auth/me')).set('Authorization', `Bearer ${forged}`).expect(401);
    });

    it('rejects tokens signed with the wrong secret', async () => {
      const { SignJWT } = require('jose');
      const bad = await new SignJWT({ sub: 'x', role: 'resident', type: 'access' })
        .setProtectedHeader({ alg: 'HS256' })
        .setIssuer('always-together-api')
        .setExpirationTime('5m')
        .sign(new TextEncoder().encode('attacker-secret-that-is-definitely-long-enough'));
      await api().get(url('/auth/me')).set('Authorization', `Bearer ${bad}`).expect(401);
    });

    it('rotates refresh tokens and revokes the family on reuse', async () => {
      const { refreshToken } = await registerAndLogin('rotate@example.com');
      const r1 = await api().post(url('/auth/refresh')).send({ refreshToken }).expect(200);
      expect(r1.body.refreshToken).not.toBe(refreshToken);
      // replay the ORIGINAL token → reuse detected, whole family revoked
      await api().post(url('/auth/refresh')).send({ refreshToken }).expect(401);
      // even the legitimately rotated token must now be dead
      await api().post(url('/auth/refresh')).send({ refreshToken: r1.body.refreshToken }).expect(401);
    });
  });

  describe('SC-04 account lockout', () => {
    it('locks the account after LOGIN_MAX_ATTEMPTS failures', async () => {
      await api().post(url('/auth/register')).send({ email: 'lock@example.com', password: strongPw, fullName: 'L' }).expect(201);
      for (let i = 0; i < 5; i++) {
        await api().post(url('/auth/login')).send({ email: 'lock@example.com', password: 'Wrong-Password-9!' }).expect(401);
      }
      // correct password now rejected while locked
      await api().post(url('/auth/login')).send({ email: 'lock@example.com', password: strongPw }).expect(401);
      expect(store.users.findByEmailNormalized('lock@example.com')!.isLocked).toBe(true);
    });
  });

  describe('SC-05 rate limiting', () => {
    it('returns 429 with Retry-After beyond the window limit', async () => {
      process.env.RATE_LIMIT_MAX = '3';
      const fast = await Test.createTestingModule({ imports: [AppModule] }).compile();
      const app2 = fast.createNestApplication();
      app2.setGlobalPrefix('/api/v1');
      await app2.init();
      let last: request.Response | undefined;
      for (let i = 0; i < 6; i++) {
        last = await request(app2.getHttpServer()).get('/api/v1/health/nope');
      }
      expect(last?.status).toBe(429);
      expect(last?.headers['retry-after']).toBeDefined();
      expect(last?.body.title).toBe('Too Many Requests');
      await app2.close();
    });
  });

  describe('SC-06 authorization / consent (IDOR checks)', () => {
    it('denies location access without active consent; grants after share; ends on revoke', async () => {
      const alice = await registerAndLogin('alice@example.com');
      const bob = await registerAndLogin('bob@example.com');
      const carol = await registerAndLogin('carol@example.com');

      // Alice reports a location
      await api().post(url('/me/location')).set('Authorization', `Bearer ${alice.accessToken}`)
        .send({ lat: 45.8, lng: 16.0, accuracyMeters: 10 }).expect(201);

      // Bob has no friendship/consent → sees nothing of Alice
      let res = await api().get(url('/latest-locations')).set('Authorization', `Bearer ${bob.accessToken}`).expect(200);
      expect(res.body.items).toHaveLength(0);

      // friendship: alice -> bob, accepted
      const bobId = store.users.findByEmailNormalized('bob@example.com')!.id;
      const f = await api().post(url('/friendships')).set('Authorization', `Bearer ${alice.accessToken}`)
        .send({ targetUserId: bobId }).expect(201);
      await api().post(url(`/friendships/${f.body.id}/accept`)).set('Authorization', `Bearer ${bob.accessToken}`).expect(201);

      // consent: alice shares approximate for 60 min
      const s = await api().post(url('/location-shares')).set('Authorization', `Bearer ${alice.accessToken}`)
        .send({ recipientUserId: bobId, precision: 'approximate', durationMinutes: 60 }).expect(201);

      res = await api().get(url('/latest-locations')).set('Authorization', `Bearer ${bob.accessToken}`).expect(200);
      expect(res.body.items).toHaveLength(1);
      // precision applied: coordinates coarsened to ~5km grid
      expect(String(res.body.items[0].lat)).toMatch(/^45\.[789]00000|^45\.[0-9]*0$/);

      // Carol (not a friend, no consent) cannot read the share record either
      await api().get(url(`/location-shares/${s.body.id}`)).set('Authorization', `Bearer ${carol.accessToken}`).expect(404);

      // Bob cannot revoke Alice's consent (only owner can)
      await api().delete(url(`/location-shares/${s.body.id}`)).set('Authorization', `Bearer ${bob.accessToken}`).expect(403);

      // Alice revokes → Bob immediately loses access
      await api().delete(url(`/location-shares/${s.body.id}`)).set('Authorization', `Bearer ${alice.accessToken}`).expect(200);
      res = await api().get(url('/latest-locations')).set('Authorization', `Bearer ${bob.accessToken}`).expect(200);
      expect(res.body.items).toHaveLength(0);
    });

    it('rejects malformed bodies (validation, unknown fields)', async () => {
      const alice = await registerAndLogin('val@example.com');
      await api().post(url('/me/location')).set('Authorization', `Bearer ${alice.accessToken}`)
        .send({ lat: 999, lng: 16 }).expect(400);
      await api().post(url('/me/location')).set('Authorization', `Bearer ${alice.accessToken}`)
        .send({ lat: 45, lng: 16, evilField: 'x' }).expect(400);
    });
  });

  describe('SC-07 error hygiene', () => {
    it('all errors are RFC7807 problem+json without internals', async () => {
      const res = await api().get(url('/auth/me')).expect(401);
      expect(res.headers['content-type']).toMatch(/application\/problem\+json/);
      expect(res.body).toMatchObject({ title: expect.any(String), status: 401 });
      expect(JSON.stringify(res.body)).not.toMatch(/at .*\.ts|Error:|stack/i);
    });
  });
});
