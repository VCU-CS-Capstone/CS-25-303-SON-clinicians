export interface PaginatedResponse<T> {
  total_pages: number;
  total: number;
  data: T[];
}

export interface SiteInfo {
  branch?: string;
  build_time: string;
  commit?: string;
  commit_time?: string;
  version: string;

  features: {
    open_api_routes: boolean;
    red_cap_read_syncing: boolean;
    red_cap_write_syncing: boolean;
    scalar: boolean;
    update_participant: boolean;
  };
}
