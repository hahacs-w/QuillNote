export interface DraftMeta {
  id: string;
  title: string;
  content_file: string;
  folder_id: string | null;
  created_at: string;
  updated_at: string;
  parent_id?: string | null;
}

export interface Folder {
  id: string;
  name: string;
  parent_id: string | null;
}

export interface Tag {
  id: string;
  name: string;
}

export interface DraftLink {
  id: string;
  draft_id: string;
  url_or_path: string;
  alias: string | null;
  created_at: string;
}

export interface GlobalSearchResult {
  draft: DraftMeta;
  excerpt: string;
}
