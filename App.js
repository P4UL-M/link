import { StyleSheet, View, Text } from "react-native";
import Home from "./src/home/home";
import LoginScreen from "./src/login/login";
import RegisterScreen from "./src/register/register";
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
                        <Stack.Screen name="Login" component={LoginScreen} options={{ headerShown: false }} />
                        <Stack.Screen name="Home" component={Home} options={{ headerShown: false }} />
                        <Stack.Screen name="Register" component={RegisterScreen} options={{ headerShown: false }} />
                    </Stack.Navigator>
                </NavigationContainer>
            </AxiosProvider>
        </AuthProvider>
    );
}