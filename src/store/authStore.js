import * as Keychain from "react-native-keychain";
import { Platform } from "react-native";

async function storeAuth(accessToken, refreshToken) {
    if (Platform.OS === "native") {
        await Keychain.setGenericPassword(
            "token",
            JSON.stringify({
                accessToken,
                refreshToken,
            })
        );
    } else {
        sessionStorage.setItem(
            "token",
            JSON.stringify({
                accessToken,
                refreshToken,
            })
        );
    }
}

async function getAuth() {
    if (Platform.OS === "native") {
        const token = await Keychain.getGenericPassword();
        if (token) {
            return JSON.parse(token);
        }
    } else {
        const token = sessionStorage.getItem("token");
        if (token) {
            return JSON.parse(token);
        }
    }
}

export { storeAuth, getAuth };
