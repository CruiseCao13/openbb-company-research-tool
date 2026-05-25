export type AppInfo = {
  app_version: string;
  repo_root: string;
  reports_root: string;
  platform: string;
  studio_mode: string;
};

export type AppInfoStatus = "loading" | "connected" | "failed" | "browser-preview";
