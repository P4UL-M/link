import React, { useContext, useEffect, useRef, useState } from 'react';
import stylesheet from './style';
import {
    KeyboardAvoidingView,
    Text,
    View,
    Platform,
    SafeAreaView,
    ActivityIndicator,
    FlatList,
    TouchableOpacity
} from 'react-native';
import { Button } from 'react-native-elements';
import { AuthContext } from '../context/AuthContext';
import { logout, refresh } from '../services/auth.service';
import { useTheme, useIsFocused } from '@react-navigation/native';
import { createDrawerNavigator } from '@react-navigation/drawer';
import { useApolloClient, gql, useQuery } from '@apollo/client';
import InputBar from '../InputBar/inputbar';
import MyChannel from '../Channel/channel';
import { AxiosContext } from '../context/AxiosContext';
import KeyboardDismissView from '../KeyboardDismissView/KeyboardDismissView';
import SimpleIcon from 'react-native-vector-icons/SimpleLineIcons';
import IonIcon from 'react-native-vector-icons/Ionicons';
import MaterialIcon from 'react-native-vector-icons/MaterialIcons';

function ListConnectedUsers() {
    const authContext = useContext(AuthContext);

    const flatListRef = useRef(null);

    const { data } = useQuery(gql`query {
        Users {
            pseudo
            publicKey
            isConnected
        }
    }`, {
        context: {
            headers: {
                'Authorization': `Bearer ${authContext.getAccessToken()}`,
            },
        },
        pollInterval: 5000,
    });

    return (
        <SafeAreaView style={{ flex: 1, alignItems: 'center', justifyContent: 'center' }}>
            <FlatList
                data={data?.Users || []}
                renderItem={({item}) => (
                    <TouchableOpacity>
                        <View>
                            <Text>{item?.pseudo}</Text>
                            <Text>{item?.isConnected ? 'connected' : 'deconnected'}</Text>
                        </View>
                    </TouchableOpacity>
                )}
                keyExtractor={item => item.pseudo + '#' + item.publicKey}
                ListEmptyComponent={() => (
                    <View>
                        <ActivityIndicator size="large" color="#0000ff" />
                    </View>
                )}
                extraData={data}
                style={{flexGrow: 1, flex:1}}
                ref={flatListRef}
                onLayout={() => {
                    if (flatListRef.current && data) {
                        flatListRef.current.scrollToEnd({animated: true});
                    }
                } }
                onContentSizeChange={() => {
                    if (flatListRef.current && data) {
                        flatListRef.current.scrollToEnd({animated: true});
                    }
                } }
            />
        </SafeAreaView>
    );
}

function Profil({navigation}) {
    const authContext = useContext(AuthContext);
    const { colors } = useTheme();
    const styles = stylesheet(colors);
    const client = useApolloClient();

    const [profil, setProfil] = useState({
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
        }
    }
    useEffect(() => {
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
                <MaterialIcon
                    name="logout"
                    size={30}
                    color={colors.text}
                    onPress={() => {
                        logout(authContext);
                        navigation.navigate('Login');
                    }}
                />
            </View>
        </View>
    );
}

const LeftDrawer = createDrawerNavigator();

const RightDrawer = createDrawerNavigator();

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
            <LeftDrawer.Screen name="Home" component={Home}/>
        </LeftDrawer.Navigator>
    );
}


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
            drawerContent={(props) => <ListConnectedUsers {...props} />}
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
    const { colors } = useTheme();
    const styles = stylesheet(colors);

    return (
        <KeyboardAvoidingView
            behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
            style={styles.container}>
            <KeyboardDismissView>
                <View style={styles.inner}>
                    <View>
                        <SafeAreaView />
                        <View style={{
                            flexDirection:'row',
                            alignItems: 'stretch',
                        }}>
                            <TouchableOpacity style={{
                                padding: 10
                            }}
                            onPress={() => navigation.getParent('LeftDrawer').openDrawer('ProfilSideMenu')}>
                                <SimpleIcon
                                    name='menu'
                                    size={22} />
                            </TouchableOpacity>
                            <Text style={[styles.header, {flexGrow:1}]}>Link</Text>
                            <TouchableOpacity style={{
                                padding: 10
                            }}
                            onPress={() => navigation.getParent('RightDrawer').openDrawer()}>
                                <IonIcon
                                    name='people'
                                    size={22} />
                            </TouchableOpacity>
                        </View>
                    </View>
                    <MyChannel />
                </View>
            </KeyboardDismissView>
            <View>
                <InputBar />
                <SafeAreaView />
            </View>
        </KeyboardAvoidingView>
    );
}

export default HomeScreen;
