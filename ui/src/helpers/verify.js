import axios from "axios";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL

export async function putVerifyRole(guildId, roleId, pattern, accessToken) {
  return await axios.put(`${VITE_KB_API_URL}/guilds/${guildId}/verify/roles/${roleId}`, {
    'pattern': pattern
  },
    {
      headers: {
        'Authorization': 'Discord ' + accessToken
      }
    }
  )
}

export async function deleteVerifyRole(guildId, roleId, accessToken) {
  return await axios.delete(`${VITE_KB_API_URL}/guilds/${guildId}/verify/roles/${roleId}`,
    {
      headers: {
        'Authorization': 'Discord ' + accessToken
      }
    }
  )
}