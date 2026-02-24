export interface CryptoAddress {
  id: string;
  profile_id: string;
  network: string;
  address: string;
  label: string;
  created_at: string;
}

export interface ProfileLink {
  id: string;
  profile_id: string;
  url: string;
  label: string;
  platform: string;
  display_order: number;
  created_at: string;
}

export interface ProfileSection {
  id: string;
  profile_id: string;
  section_type: string;
  title: string;
  content: string;
  display_order: number;
  created_at: string;
}

export interface ProfileSkill {
  id: string;
  profile_id: string;
  skill: string;
  created_at: string;
}

export interface Endorsement {
  id: string;
  endorsee_id: string;
  endorser_username: string;
  message: string;
  signature: string;
  verified: boolean;
  created_at: string;
}

export interface Profile {
  id: string;
  username: string;
  display_name: string;
  tagline: string;
  bio: string;
  third_line: string;
  avatar_url: string;
  theme: string;
  particle_effect: string;
  particle_enabled: boolean;
  particle_seasonal: boolean;
  pubkey: string;
  profile_score: number;
  view_count: number;
  created_at: string;
  updated_at: string;
  crypto_addresses: CryptoAddress[];
  links: ProfileLink[];
  sections: ProfileSection[];
  skills: ProfileSkill[];
  endorsements: Endorsement[];
}
