import { StyleSheet, View, Text } from "react-native";
import InputBar from "./Inputbar";
import LoginScreen from "./src/login/login";
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';

const Stack = createNativeStackNavigator();

export default function App() {
    return (
        <NavigationContainer>
            <Stack.Navigator initialRouteName="Login">
                <Stack.Screen name="Login" component={LoginScreen} options={{ headerShown: true }} />
                <Stack.Screen name="InputBar" component={InputBar} options={{ headerShown: true }} />
            </Stack.Navigator>
        </NavigationContainer>
        // <View style={styles.container}>
        //     <Text style={styles.text}>Open up App.js to start working on your app!</Text>
        //     <InputBar />

    // </View>
    );
}

const styles = StyleSheet.create({
    container: {
        flex: 1,
        backgroundColor: "#fff",
        alignItems: "center",
        justifyContent: "center",
    },
    text: {
        flex: 1,
        fontSize: 30,
    }
});
