/**
 * RFC 7807 Problem Details error body — mandated by the MVP API contract
 * (docs/reference/api-contract.md). All error responses use this shape so
 * clients never receive stack traces or internal details (SC-07).
 */
export interface ProblemDetails {
  type: string;
  title: string;
  status: number;
  detail?: string;
  instance?: string;
  code?: string;
  fields?: Record<string, string[]>;
}

export function problemDetails(opts: {
  type?: string;
  title: string;
  status: number;
  detail?: string;
  instance?: string;
  code?: string;
  fields?: Record<string, string[]>;
}): ProblemDetails {
  return {
    type: opts.type ?? 'https://always-together.local/errors',
    title: opts.title,
    status: opts.status,
    ...(opts.detail !== undefined && { detail: opts.detail }),
    ...(opts.instance !== undefined && { instance: opts.instance }),
    ...(opts.code !== undefined && { code: opts.code }),
    ...(opts.fields !== undefined && { fields: opts.fields }),
  };
}
