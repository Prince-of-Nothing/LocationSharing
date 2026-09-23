import { randomUUID } from 'node:crypto';

/**
 * Domain model — mirrors docs/reference/domain-data-model.md and the
 * prisma/schema.prisma definitions. Kept as plain interfaces so the service
 * layer can run against either PostgreSQL (Prisma) or the in-memory
 * repository used by the security test-suite.
 */

export type UserRole = 'resident' | 'administrator';
export type FriendshipStatus = 'pending' | 'accepted' | 'declined' | 'blocked';
export type SharePrecision = 'exact' | 'approximate' | 'city_only';
export type CheckInState =
  | 'pending'
  | 'reminder_sent'
  | 'grace_period'
  | 'completed'
  | 'cancelled'
  | 'missed';

export interface User {
  id: string;
  email: string;
  emailNormalized: string;
  passwordHash: string;
  fullName: string;
  role: UserRole;
  isActive: boolean;
  isLocked: boolean;
  failedLoginAttempts: number;
  mustChangePassword: boolean;
  createdAt: Date;
  updatedAt?: Date;
  lastLoginAt?: Date;
}

export interface RefreshTokenRecord {
  id: string;
  tokenHash: string; // SHA-256 of the opaque refresh token; raw value never stored
  userId: string;
  familyId: string; // rotation family for reuse detection
  expiresAt: Date;
  revokedAt?: Date;
  replacedBy?: string;
  createdAt: Date;
}

export interface AuditEvent {
  id: string;
  userId?: string;
  action: string; // e.g. auth.login.success, consent.share.created
  outcome: 'success' | 'failure' | 'denied';
  ip?: string;
  meta?: Record<string, unknown>;
  createdAt: Date;
}

export interface Friendship {
  id: string;
  requesterId: string;
  receiverId: string;
  status: FriendshipStatus;
  createdAt: Date;
  respondedAt?: Date;
}

/** Directional, time-boxed consent to share location with one recipient. */
export interface LocationShare {
  id: string;
  ownerId: string; // the user whose location is shared
  recipientId: string; // the user allowed to see it
  precision: SharePrecision;
  purpose?: string;
  isActive: boolean;
  expiresAt?: Date;
  createdAt: Date;
}

/** Latest-known location only. MVP stores no traversal history (privacy). */
export interface LatestLocation {
  id: string;
  ownerId: string;
  lat: number;
  lng: number;
  accuracyMeters?: number;
  batteryLevel?: number;
  observedAt: Date;
  createdAt: Date;
}

export interface CheckIn {
  id: string;
  userId: string;
  type: 'manual' | 'scheduled' | 'trip';
  state: CheckInState;
  dueAt?: Date;
  graceEndsAt?: Date;
  acknowledgedBy?: string[];
  completedAt?: Date;
  createdAt: Date;
}

export function newId(): string {
  return randomUUID();
}
