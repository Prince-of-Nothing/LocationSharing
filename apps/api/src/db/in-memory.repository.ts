import {
  AuditEvent,
  CheckIn,
  Friendship,
  LatestLocation,
  LocationShare,
  RefreshTokenRecord,
  User,
  newId,
} from '../common/domain';

/**
 * Repository interface used by all services. Production binds the Prisma
 * implementation; the security test-suite and demo mode bind this in-memory
 * implementation so the whole API can run without external infrastructure
 * (useful inside constrained CI containers without Docker-in-Docker).
 */
export interface DataStore {
  users: {
    create(u: Omit<User, 'id' | 'createdAt'>): User;
    findById(id: string): User | undefined;
    findByEmailNormalized(email: string): User | undefined;
    update(id: string, patch: Partial<User>): User | undefined;
    list(): User[];
  };
  refreshTokens: {
    create(r: Omit<RefreshTokenRecord, 'id' | 'createdAt'>): RefreshTokenRecord;
    findByHash(tokenHash: string): RefreshTokenRecord | undefined;
    update(id: string, patch: Partial<RefreshTokenRecord>): void;
    revokeAllForUser(userId: string): void;
    revokeFamily(familyId: string): void;
  };
  audit: { append(e: Omit<AuditEvent, 'id' | 'createdAt'>): AuditEvent; list(): AuditEvent[] };
  friendships: {
    create(f: Omit<Friendship, 'id' | 'createdAt'>): Friendship;
    findById(id: string): Friendship | undefined;
    between(a: string, b: string): Friendship | undefined;
    acceptedWith(userId: string): Friendship[];
    update(id: string, patch: Partial<Friendship>): Friendship | undefined;
  };
  shares: {
    create(s: Omit<LocationShare, 'id' | 'createdAt'>): LocationShare;
    findById(id: string): LocationShare | undefined;
    activeFor(ownerId: string, recipientId: string, now?: Date): LocationShare | undefined;
    activeReceivedBy(recipientId: string, now?: Date): LocationShare[];
    update(id: string, patch: Partial<LocationShare>): LocationShare | undefined;
    allForPair(a: string, b: string): LocationShare[];
  };
  locations: {
    upsertLatest(l: Omit<LatestLocation, 'id' | 'createdAt'>): LatestLocation;
    latestFor(ownerId: string): LatestLocation | undefined;
  };
  checkins: {
    create(c: Omit<CheckIn, 'id' | 'createdAt'>): CheckIn;
    findById(id: string): CheckIn | undefined;
    update(id: string, patch: Partial<CheckIn>): CheckIn | undefined;
    listForUser(userId: string): CheckIn[];
  };
}

class Table<T extends { id: string }> {
  protected rows = new Map<string, T>();
  insert(row: Omit<T, 'id'> & { id?: string }): T {
    const full = { ...row, id: row.id ?? newId() } as T;
    this.rows.set(full.id, full);
    return full;
  }
  byId(id: string): T | undefined {
    return this.rows.get(id);
  }
  patch(id: string, changes: Partial<T>): T | undefined {
    const cur = this.rows.get(id);
    if (!cur) return undefined;
    const next = { ...cur, ...changes };
    this.rows.set(id, next);
    return next;
  }
  all(): T[] {
    return [...this.rows.values()];
  }
}

export class InMemoryStore extends Table<User> implements DataStore {
  private rt = new Table<RefreshTokenRecord>();
  private au = new Table<AuditEvent>();
  private fr = new Table<Friendship>();
  private sh = new Table<LocationShare>();
  private lo = new Table<LatestLocation>();
  private ci = new Table<CheckIn>();

  users = {
    create: (u: Omit<User, 'id' | 'createdAt'>) =>
      this.insert({ ...u, createdAt: new Date() }),
    findById: (id: string) => this.byId(id),
    findByEmailNormalized: (email: string) =>
      this.all().find((u) => u.emailNormalized === email),
    update: (id: string, patch: Partial<User>) => this.patch(id, patch),
    list: () => this.all(),
  };

  refreshTokens = {
    create: (r: Omit<RefreshTokenRecord, 'id' | 'createdAt'>) =>
      this.rt.insert({ ...r, createdAt: new Date() }),
    findByHash: (tokenHash: string) =>
      this.rt.all().find((t) => t.tokenHash === tokenHash),
    update: (id: string, patch: Partial<RefreshTokenRecord>) => {
      this.rt.patch(id, patch);
    },
    revokeAllForUser: (userId: string) => {
      for (const t of this.rt.all())
        if (t.userId === userId && !t.revokedAt)
          this.rt.patch(t.id, { revokedAt: new Date() });
    },
    revokeFamily: (familyId: string) => {
      for (const t of this.rt.all())
        if (t.familyId === familyId && !t.revokedAt)
          this.rt.patch(t.id, { revokedAt: new Date() });
    },
  };

  audit = {
    append: (e: Omit<AuditEvent, 'id' | 'createdAt'>) =>
      this.au.insert({ ...e, createdAt: new Date() }),
    list: () => this.au.all(),
  };

  friendships = {
    create: (f: Omit<Friendship, 'id' | 'createdAt'>) =>
      this.fr.insert({ ...f, createdAt: new Date() }),
    findById: (id: string) => this.fr.byId(id),
    between: (a: string, b: string) =>
      this.fr.all().find(
        (f) =>
          (f.requesterId === a && f.receiverId === b) ||
          (f.requesterId === b && f.receiverId === a),
      ),
    acceptedWith: (userId: string) =>
      this.fr.all().filter(
        (f) => f.status === 'accepted' && (f.requesterId === userId || f.receiverId === userId),
      ),
    update: (id: string, patch: Partial<Friendship>) => this.fr.patch(id, patch),
  };

  shares = {
    create: (s: Omit<LocationShare, 'id' | 'createdAt'>) =>
      this.sh.insert({ ...s, createdAt: new Date() }),
    findById: (id: string) => this.sh.byId(id),
    activeFor: (ownerId: string, recipientId: string, now = new Date()) =>
      this.sh.all().find(
        (s) =>
          s.ownerId === ownerId &&
          s.recipientId === recipientId &&
          s.isActive &&
          (!s.expiresAt || s.expiresAt > now),
      ),
    activeReceivedBy: (recipientId: string, now = new Date()) =>
      this.sh.all().filter(
        (s) => s.recipientId === recipientId && s.isActive && (!s.expiresAt || s.expiresAt > now),
      ),
    update: (id: string, patch: Partial<LocationShare>) => this.sh.patch(id, patch),
    allForPair: (a: string, b: string) =>
      this.sh.all().filter(
        (s) =>
          (s.ownerId === a && s.recipientId === b) ||
          (s.ownerId === b && s.recipientId === a),
      ),
  };

  locations = {
    upsertLatest: (l: Omit<LatestLocation, 'id' | 'createdAt'>) => {
      const existing = this.lo.all().find((x) => x.ownerId === l.ownerId);
      if (existing) return this.lo.patch(existing.id, { ...l })!;
      return this.lo.insert({ ...l, createdAt: new Date() });
    },
    latestFor: (ownerId: string) =>
      this.lo.all().find((l) => l.ownerId === ownerId),
  };

  checkins = {
    create: (c: Omit<CheckIn, 'id' | 'createdAt'>) =>
      this.ci.insert({ ...c, createdAt: new Date() }),
    findById: (id: string) => this.ci.byId(id),
    update: (id: string, patch: Partial<CheckIn>) => this.ci.patch(id, patch),
    listForUser: (userId: string) => this.ci.all().filter((c) => c.userId === userId),
  };
}

export const DATA_STORE = Symbol('DATA_STORE');
