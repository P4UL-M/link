import React, { useContext } from 'react';
import stylesheet from './style';
import {
    Keyboard,
    KeyboardAvoidingView,
    Text,
    TouchableWithoutFeedback,
    View,
    Platform,
} from 'react-native';
import { Button } from 'react-native-elements';
import { AuthContext } from '../context/AuthContext';
import { AxiosContext } from '../context/AxiosContext';
import { logout } from '../services/auth.service';
import { useTheme } from '@react-navigation/native';
import { createDrawerNavigator } from '@react-navigation/drawer';

function RightDrawerContent() {
    return (
        <View style={{ flex: 1, alignItems: 'center', justifyContent: 'center' }}>
            <Text>This is the right drawer</Text>
        </View>
    );
}

const LeftDrawer = createDrawerNavigator();

function LeftDrawerScreen() {
    return (
        <LeftDrawer.Navigator
            id="LeftDrawer"
            screenOptions={{ 
                drawerPosition: 'left', 
                headerShown: false, 
                drawerHideStatusBarOnOpen: true,
                drawerStatusBarAnimation: 'none', 
            }}>
            <LeftDrawer.Screen name="Home" component={Home} />
        </LeftDrawer.Navigator>
    );
}

const RightDrawer = createDrawerNavigator();

function HomeScreen() {
    return (
        <RightDrawer.Navigator
            hideStatusBar={true}
            id="RightDrawer"
            drawerContent={(props) => <RightDrawerContent {...props} />}
            screenOptions={{
                drawerPosition: 'right',
                headerShown: false,
                drawerHideStatusBarOnOpen: true,
                drawerStatusBarAnimation: 'none',
            }
            }>
            <RightDrawer.Screen name="HomeDrawer" component={LeftDrawerScreen} />
        </RightDrawer.Navigator>
    );
}


function Home({navigation}) {
    const authContext = useContext(AuthContext);
    const { authAxios } = useContext(AxiosContext);

    const { colors } = useTheme();
    const styles = stylesheet(colors);

    async function getUser() {
        const response = await authAxios.post('graphql', {
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

    return (
        <KeyboardAvoidingView
            behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
            style={styles.container}
        >
            <TouchableWithoutFeedback
                onPress={Platform.select({
                    native: Keyboard.dismiss,
                    web: () => null,
                })}
            >
                <View style={styles.inner}>
                    <Text style={styles.header}>Header</Text>
                    <View style={styles.btnContainer}>
                        <Button title="Submit" onPress={getUser} buttonStyle={styles.btn} />
                    </View>
                    <View style={styles.btnContainer}>
                        <Button
                            title="Logout"
                            onPress={() => {
                                logout(authContext);
                                navigation.navigate('Login');
                            }}
                            buttonStyle={styles.btn}
                        />
                    </View>
                </View>
            </TouchableWithoutFeedback>
        </KeyboardAvoidingView>
    );
}

export default HomeScreen;
