import React, { useContext } from "react";
import styles from "../login/style";
import {
    Keyboard,
    KeyboardAvoidingView,
    Text,
    TouchableWithoutFeedback,
    View,
    Platform,
} from "react-native";
import { Button } from "react-native-elements";
import tw from "../../lib/tailwind"; // or, if no custom config: `from 'twrnc'`
import { useDeviceContext } from "twrnc";
import { useNavigation } from "@react-navigation/native";
import { useIsFocused } from "@react-navigation/native";
import { AuthContext } from "../context/AuthContext";
import { AxiosContext } from "../context/AxiosContext";

const Home = () => {
    const authContext = useContext(AuthContext);
    const { authAxios, publicAxios } = useContext(AxiosContext);

    useDeviceContext(tw);
    const nav = useNavigation();
    const isFocused = useIsFocused();

    function getUser() {
        const response = authAxios.post("graphql", {
            query: `
                query {
                    whoami {
                        _id,
                        pseudo,
                        email,
                        publicKey
                    }
                }
            `,
        });

        console.log(response);
    }

    function refresh() {
        const response = publicAxios.get("auth/refresh", {
            headers: { refresh_token: authContext.authState.refreshToken },
        });

        console.log(response);
    }

    return (
        <KeyboardAvoidingView
            behavior={Platform.OS === "ios" ? "padding" : "height"}
            style={styles.container}
        >
            <TouchableWithoutFeedback
                onPress={Platform.select({
                    native: Keyboard.dismiss,
                    web: () => null,
                })}
            >
                <View style={[styles.inner, tw` bg-white dark:bg-gray-700`]}>
                    <Text style={styles.header}>Header</Text>
                    <View style={styles.btnContainer}>
                        <Button title="Submit" onPress={getUser} />
                    </View>
                    <View style={styles.btnContainer}>
                        <Button title="Submit" onPress={refresh} />
                    </View>
                </View>
            </TouchableWithoutFeedback>
        </KeyboardAvoidingView>
    );
};

export default Home;
