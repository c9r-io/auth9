import { API_BASE_URL, handleResponse } from "./client";

export interface WebAuthnCredential {
  id: string;
  credential_type: string;
  user_label?: string;
  created_at: string;
}

function arrayBufferToBase64url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary)
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
}

export function base64urlToArrayBuffer(base64url: string): ArrayBuffer {
  const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/");
  const padded = base64 + "=".repeat((4 - (base64.length % 4)) % 4);
  const binary = atob(padded);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

export const webauthnApi = {
  listPasskeys: async (
    accessToken: string
  ): Promise<{ data: WebAuthnCredential[] }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/users/me/passkeys`,
      {
        headers: { Authorization: `Bearer ${accessToken}` },
      }
    );
    return handleResponse(response);
  },

  deletePasskey: async (
    credentialId: string,
    accessToken: string
  ): Promise<{ message: string }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/users/me/passkeys/${credentialId}`,
      {
        method: "DELETE",
        headers: { Authorization: `Bearer ${accessToken}` },
      }
    );
    return handleResponse(response);
  },

  startRegistration: async (
    accessToken: string,
    label?: string
  ): Promise<{ data: unknown }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/users/me/passkeys/register/start`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${accessToken}`,
        },
        body: JSON.stringify({ label }),
      }
    );
    return handleResponse(response);
  },

  completeRegistration: async (
    credential: Credential,
    accessToken: string,
    label?: string
  ): Promise<{ message: string }> => {
    const pkCred = credential as PublicKeyCredential;
    const attestation =
      pkCred.response as AuthenticatorAttestationResponse;
    const body = {
      credential: {
        id: pkCred.id,
        rawId: arrayBufferToBase64url(pkCred.rawId),
        type: pkCred.type,
        response: {
          attestationObject: arrayBufferToBase64url(
            attestation.attestationObject
          ),
          clientDataJSON: arrayBufferToBase64url(
            attestation.clientDataJSON
          ),
        },
      },
      label,
    };
    const response = await fetch(
      `${API_BASE_URL}/api/v1/users/me/passkeys/register/complete`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${accessToken}`,
        },
        body: JSON.stringify(body),
      }
    );
    return handleResponse(response);
  },

  startAuthentication: async (): Promise<{
    data: { challenge_id: string; publicKey: unknown };
  }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/auth/webauthn/authenticate/start`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
      }
    );
    return handleResponse(response);
  },

  completeAuthentication: async (
    challengeId: string,
    credential: Credential
  ): Promise<{
    access_token: string;
    token_type: string;
    expires_in: number;
  }> => {
    const pkCred = credential as PublicKeyCredential;
    const assertion = pkCred.response as AuthenticatorAssertionResponse;
    const body = {
      challenge_id: challengeId,
      credential: {
        id: pkCred.id,
        rawId: arrayBufferToBase64url(pkCred.rawId),
        type: pkCred.type,
        response: {
          authenticatorData: arrayBufferToBase64url(
            assertion.authenticatorData
          ),
          clientDataJSON: arrayBufferToBase64url(
            assertion.clientDataJSON
          ),
          signature: arrayBufferToBase64url(assertion.signature),
          userHandle: assertion.userHandle
            ? arrayBufferToBase64url(assertion.userHandle)
            : undefined,
        },
      },
    };
    const response = await fetch(
      `${API_BASE_URL}/api/v1/auth/webauthn/authenticate/complete`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      }
    );
    return handleResponse(response);
  },
};
