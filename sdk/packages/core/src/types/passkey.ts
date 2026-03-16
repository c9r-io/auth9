export interface Passkey {
  id: string;
  name?: string;
  credentialId: string;
  createdAt: string;
  lastUsedAt?: string;
}

export interface PasskeyRegistrationOptions {
  publicKey: Record<string, unknown>;
}

export interface PasskeyRegistrationResult {
  id: string;
  rawId: string;
  type: string;
  response: Record<string, unknown>;
}

export interface PasskeyAuthStartInput {
  email?: string;
}

export interface PasskeyAuthenticationOptions {
  challengeId: string;
  publicKey: Record<string, unknown>;
}

export interface PasskeyAuthenticationResult {
  challengeId: string;
  id: string;
  rawId: string;
  type: string;
  response: Record<string, unknown>;
}
