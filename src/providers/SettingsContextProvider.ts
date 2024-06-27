import { watch, provide, reactive, InjectionKey, toRefs, ToRefs } from "vue";
import {
  BaseDirectory,
  exists,
  createDir,
  writeTextFile,
  readTextFile,
} from "@tauri-apps/api/fs";

export const SettingsContextStateKey: InjectionKey<
  ToRefs<SettingsContextState>
> = Symbol("SettingsContextState");

export interface ContextSettings {
  context: string;
  namespaces: string[];
}

export interface SettingsContextState {
  settings: {
    lastContext: string | null;
    lastNamespace: string | null;
    tabProvider: {
      height: number;
    };
    shell: {
      executable: string;
    };
    structuredLogViewer: {
      enabled: boolean;
    };
    contextSettings: ContextSettings[];
    collapsedNavigationGroups: string[];
    appearance: {
      colorScheme: "auto" | "light" | "dark";
    };
    updates: {
      checkOnStartup: boolean;
    };
  };
}

export default {
  name: "SettingsContextProvider",
  async setup() {
    const settingsFile = "settings.json";

    const state: SettingsContextState = reactive({
      settings: {
        lastContext: null,
        lastNamespace: null,
        tabProvider: {
          height: 50,
        },
        structuredLogViewer: {
          enabled: false,
        },
        shell: {
          executable: "/bin/sh",
        },
        contextSettings: [],
        collapsedNavigationGroups: [],
        appearance: {
          colorScheme: "auto",
        },
        updates: {
          checkOnStartup: true,
        },
      },
    });
    provide(SettingsContextStateKey, toRefs(state));

    const save = async () => {
      if (!(await exists(settingsFile, { dir: BaseDirectory.AppConfig }))) {
        if (!(await exists("", { dir: BaseDirectory.AppConfig }))) {
          await createDir("", { dir: BaseDirectory.AppConfig });
        }
      }

      await writeTextFile(settingsFile, JSON.stringify(state.settings), {
        dir: BaseDirectory.AppConfig,
      });
    };

    if (await exists(settingsFile, { dir: BaseDirectory.AppConfig })) {
      const fileContents = await readTextFile(settingsFile, {
        dir: BaseDirectory.AppConfig,
      });

      // Merge initial state with file contents
      state.settings = { ...state.settings, ...JSON.parse(fileContents) };
    }

    watch(state, (newState) => {
      console.log("Settings changed...", newState);
      save();
    });
  },
  render(): any {
    return this.$slots.default();
  },
};
