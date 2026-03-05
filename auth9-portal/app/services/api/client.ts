// Shared API client utilities

export const API_BASE_URL =
  process.env.AUTH9_CORE_URL || "http://localhost:8080";

export interface ApiError {
  error: string;
  message: string;
  details?: unknown;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    page: number;
    per_page: number;
    total: number;
    total_pages: number;
  };
}

export class ApiResponseError extends Error {
  code: string;
  details?: unknown;
  status: number;
  constructor(error: ApiError, status: number) {
    super(error.message);
    this.name = "ApiResponseError";
    this.code = error.error;
    this.details = error.details;
    this.status = status;
  }
}

export async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const error: ApiError = await response.json().catch(() => ({
      error: "unknown",
      message: response.statusText,
    }));
    throw new ApiResponseError(error, response.status);
  }
  return response.json();
}

export function getHeaders(accessToken?: string): HeadersInit {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  if (accessToken) {
    headers["Authorization"] = `Bearer ${accessToken}`;
  }
  return headers;
}
