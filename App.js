import { StyleSheet, View, Text } from "react-native";
import InputBar from "./Inputbar";
import LoginScreen from "./src/login/login";
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { AuthProvider } from "./src/context/AuthContext";
import { AxiosProvider } from "./src/context/AxiosContext";


const Stack = createNativeStackNavigator();

export default function App() {
    return (
        <AuthProvider>
            <AxiosProvider>
                <NavigationContainer>
                    <Stack.Navigator initialRouteName="Login">
                        <Stack.Screen name="Login" component={LoginScreen} options={{ headerShown: true }} />
                        <Stack.Screen name="InputBar" component={InputBar} options={{ headerShown: true }} />
                    </Stack.Navigator>
                </NavigationContainer>
            </AxiosProvider>
        </AuthProvider>
    );
}