export interface PersonaConfig {
  id: string;
  name: string;
  name_en: string;
  grade: string;
  race: string;
  class: string;
  sub_class: string;
  system_prompt: string;
  greeting: string;
  raw_json: string;
  created_at: string;
}


export type PersonaError =
  | { Database: string }
  | { Archive: string }
  | { NotFound: string }
  | { Unknown: string };
