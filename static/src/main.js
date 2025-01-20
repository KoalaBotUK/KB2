import { createApp } from 'vue';
import App from './App.vue';
import './assets/main.css';

import { library } from '@fortawesome/fontawesome-svg-core';
import { fas } from '@fortawesome/free-solid-svg-icons';
import { fab } from '@fortawesome/free-brands-svg-icons';
import {FontAwesomeIcon} from "@fortawesome/vue-fontawesome";

library.add(fas, fab);

createApp(App)
    .component("fa", FontAwesomeIcon)
    .mount('#app');
