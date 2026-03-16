export interface Organization {
  id: string;
  name: string;
  slug: string;
  domain: string;
  logoUrl?: string;
  status: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreateOrganizationInput {
  name: string;
  slug: string;
  domain: string;
  logoUrl?: string;
}
