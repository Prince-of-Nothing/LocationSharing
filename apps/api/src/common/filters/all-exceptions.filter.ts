import {
  ArgumentsHost,
  Catch,
  ExceptionFilter,
  HttpException,
  HttpStatus,
} from '@nestjs/common';
import { Request, Response } from 'express';
import { problemDetails } from '../problem-details';

/**
 * SC-07 — Global exception filter: every error leaves the API as RFC 7807
 * Problem Details. Unknown errors are logged server-side but clients only
 * ever see a generic 500 (no stack traces, no SQL fragments, no internals).
 */
@Catch()
export class AllExceptionsFilter implements ExceptionFilter {
  catch(exception: unknown, host: ArgumentsHost): void {
    const ctx = host.switchToHttp();
    const response = ctx.getResponse<Response>();
    const request = ctx.getRequest<Request>();

    let status = HttpStatus.INTERNAL_SERVER_ERROR;
    let body = problemDetails({
      title: 'Internal Server Error',
      status,
      detail: 'An unexpected error occurred.',
      instance: request.url,
      code: 'INTERNAL',
    });

    if (exception instanceof HttpException) {
      status = exception.getStatus();
      const res = exception.getResponse();
      if (typeof res === 'string') {
        body = problemDetails({ title: res, status, instance: request.url });
      } else if (typeof res === 'object' && res !== null) {
        const r = res as Record<string, unknown>;
        if (Array.isArray(r.message)) {
          // class-validator payload (ValidationPipe)
          body = problemDetails({
            title: 'Validation Failed',
            status,
            instance: request.url,
            code: 'VALIDATION_FAILED',
            detail: 'Request body or parameters failed validation.',
            fields: { detail: r.message.map(String) },
          });
        } else {
          body = problemDetails({
            title: String(r.title ?? exception.name),
            status,
            instance: request.url,
            code: r.code ? String(r.code) : undefined,
            detail: r.detail ? String(r.detail) : undefined,
            fields: r.fields as Record<string, string[]> | undefined,
          });
        }
      }
    }

    if (status >= 500) {
      // eslint-disable-next-line no-console
      console.error(`[error] ${request.method} ${request.url}`, exception);
    }

    response.status(status).type('application/problem+json').json(body);
  }
}
