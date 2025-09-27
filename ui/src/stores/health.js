import axios from "axios";
import {User} from "./user.js";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL

let user = User.loadCache();

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