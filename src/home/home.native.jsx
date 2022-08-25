import React, { useContext, useEffect } from 'react';
import stylesheet from './style';
import {
    KeyboardAvoidingView,
    Text,
    View,
    Platform,
    SafeAreaView,
} from 'react-native';
import { Button } from 'react-native-elements';
import { AuthContext } from '../context/AuthContext';
import { logout, refresh } from '../services/auth.service';
import { useTheme, useIsFocused } from '@react-navigation/native';
import { createDrawerNavigator } from '@react-navigation/drawer';
import { useApolloClient, gql } from '@apollo/client';
import InputBar from '../InputBar/inputbar';
import MyChannel from '../Channel/channel';
import { AxiosContext } from '../context/AxiosContext';
import KeyboardDismissView from '../KeyboardDismissView/KeyboardDismissView';

function RightDrawerContent() {
    return (
        <View style={{ flex: 1, alignItems: 'center', justifyContent: 'center' }}>
            <Text>This is the right drawer</Text>
        </View>
    );
}

function Profil({navigation}) {
    const authContext = useContext(AuthContext);
    const { colors } = useTheme();
    const styles = stylesheet(colors);
    const client = useApolloClient();

    const [profil, setProfil] = React.useState({
        pseudo: '',
        email: '',
        publicKey: '',
    });

    async function getUser() {
        const response = await client.query({
            query: gql`query whoami {
                whoami {
                    _id,
                    pseudo,
                    email,
                    publicKey
                }
            }`,
            options: {
                context: {
                    headers: {
                        'Authorization': `Bearer ${authContext.getAccessToken()}`,
                    },
                },
            },
        });
        if (response) {
            setProfil(response.data.whoami);
            return response.data;
        }
    }
    React.useEffect(() => {
        getUser();
    } , [authContext.authState]);

    return (
        <View style={{ flex: 1, alignItems: 'center', justifyContent: 'center' }}>
            <Text>This is your profil :</Text>
            <Text>{profil.pseudo}#{profil.publicKey}</Text>
            <Text>{profil.email}</Text>
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
    );
}

const LeftDrawer = createDrawerNavigator();

function LeftDrawerScreen() {
    return (
        <LeftDrawer.Navigator
            id="LeftDrawer"
            drawerContent={(props) => <Profil {...props} />}
            screenOptions={{ 
                drawerPosition: 'left', 
                headerShown: false, 
                drawerHideStatusBarOnOpen: true,
                drawerStatusBarAnimation: 'none', 
            }}>
            <LeftDrawer.Screen name="Home2" component={Home}/>
        </LeftDrawer.Navigator>
    );
}

const RightDrawer = createDrawerNavigator();

function HomeScreen({navigation}) {
    const authContext = useContext(AuthContext);
    const { publicAxios } = useContext(AxiosContext);
    const isFocused = useIsFocused();

    useEffect(() => {
        if (isFocused) {
            // clear on load
            refresh(publicAxios, authContext).then((state) => {
                if (!state) {
                    navigation.navigate('Login');
                }
            });
        }
    }, [isFocused]);

    return (
        <RightDrawer.Navigator
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

function Home() {
    const { colors } = useTheme();
    const styles = stylesheet(colors);

    return (
        <KeyboardAvoidingView
            behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
            style={styles.container}>
            <KeyboardDismissView>
                <View style={[styles.inner, {backgroundColor: 'purple'}]}>
                    <View style={{backgroundColor: 'green'}}>
                        <SafeAreaView />
                        <Text style={[styles.header, {backgroundColor: 'yellow'}]}>Header</Text>
                    </View>
                    <MyChannel />
                </View>
            </KeyboardDismissView>
            <View style={{backgroundColor: 'green'}}>
                <InputBar />
                <SafeAreaView />
            </View>
        </KeyboardAvoidingView>
    );
}

export default HomeScreen;
