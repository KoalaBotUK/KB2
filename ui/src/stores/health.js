import axios from "axios";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL

export class Health {
  status

  constructor(status) {
    this.status = status
  }

  toJson() {
    return {
      'status': this.status
    }
  }

  static fromJson(json) {
    let health = new Health()
    health.status = json['status']
    return health
  }

  static async loadHealth() {
    // Used to warm up backend
    let r = await axios.get(`${VITE_KB_API_URL}/health`, {});

    return Health.fromJson(r.data)
  }
}