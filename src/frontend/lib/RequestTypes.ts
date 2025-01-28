export interface PaginatedResponse<T> {
  total_pages: number;
  total: number;
  data: T[];
}
