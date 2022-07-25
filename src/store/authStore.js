import * as SecureStore from "expo-secure-store";
import { Platform } from "react-native";

async function storeAuth(accessToken, refreshToken) {
    if (Platform.OS === "ios" || Platform.OS === "android") {
        const data = JSON.stringify({
            accessToken,
            refreshToken,
        });
        await SecureStore.setItemAsync("token", data);
    } else {
        await sessionStorage.setItem(
            "token",
            JSON.stringify({
                accessToken,
                refreshToken,
            })
        );
    }
}

async function getAuth() {
    if (Platform.OS === "ios" || Platform.OS === "android") {
        const token = await SecureStore.getItemAsync("token");
        if (token) {
            const data = JSON.parse(token);
            return data;
        }
    } else {
        const token = sessionStorage.getItem("token");
        if (token) {
            return JSON.parse(token);
        }
    }
}

export { storeAuth, getAuth };
