import React, { useContext, useEffect } from 'react';
import stylesheet from './style';
import {
    Keyboard,
    KeyboardAvoidingView,
    Text,
    TouchableWithoutFeedback,
    View,
    Platform,
    FlatList,
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
                        'Authorization': `Bearer ${authContext.token}`,
                    },
                },
            },
        });
        //console.log(response);
        setProfil(response.data.whoami);
        return response.data;
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
            <TouchableWithoutFeedback
                onPress={Platform.select({
                    native: Keyboard.dismiss,
                    web: () => null,
                })}
            >
                <View style={[styles.inner, {backgroundColor: 'purple'}]}>
                    <View style={{backgroundColor: 'green'}}>
                        <SafeAreaView />
                        <Text style={[styles.header, {backgroundColor: 'yellow'}]}>Header</Text>
                    </View>
                    {/* list of 50 elements */}
                    {/* <FlatList data={[
                        { title: 'Title Text', key: 'item1' }, 
                        { title: 'Title Text', key: 'item2' }, 
                        { title: 'Title Text', key: 'item3' }, 
                        { title: 'Title Text', key: 'item4' }, 
                        { title: 'Title Text', key: 'item5' },
                        { title: 'Title Text', key: 'item6' },
                        { title: 'Title Text', key: 'item7' },
                        { title: 'Title Text', key: 'item8' },
                        { title: 'Title Text', key: 'item9' },
                        { title: 'Title Text', key: 'item10' },
                        { title: 'Title Text', key: 'item11' },
                        { title: 'Title Text', key: 'item12' },
                        { title: 'Title Text', key: 'item13' },
                        { title: 'Title Text', key: 'item14' },
                        { title: 'Title Text', key: 'item15' },
                        { title: 'Title Text', key: 'item16' },
                        { title: 'Title Text', key: 'item17' },
                        { title: 'Title Text', key: 'item18' },
                        { title: 'Title Text', key: 'item19' },
                        { title: 'Title Text', key: 'item20' },
                        { title: 'Title Text', key: 'item21' },
                        { title: 'Title Text', key: 'item22' },
                        { title: 'Title Text', key: 'item23' },
                        { title: 'Title Text', key: 'item24' },
                        { title: 'Title Text', key: 'item25' },
                        { title: 'Title Text', key: 'item26' },
                        { title: 'Title Text', key: 'item27' },
                        { title: 'Title Text', key: 'item28' },
                        { title: 'Title Text', key: 'item29' },
                        { title: 'Title Text', key: 'item30' },
                        { title: 'Title Text', key: 'item31' },
                        { title: 'Title Text', key: 'item32' },
                        { title: 'Title Text', key: 'item33' },
                        { title: 'Title Text', key: 'item34' },
                        { title: 'Title Text', key: 'item35' },
                        { title: 'Title Text', key: 'item36' },
                        { title: 'Title Text', key: 'item37' },
                        { title: 'Title Text', key: 'item38' },
                        { title: 'Title Text', key: 'item39' },
                        { title: 'Title Text', key: 'item40' },
                        { title: 'Title Text', key: 'item41' },
                        { title: 'Title Text', key: 'item42' },
                        { title: 'Title Text', key: 'item43' },
                        { title: 'Title Text', key: 'item44' },
                        { title: 'Title Text', key: 'item45' },
                        { title: 'Title Text', key: 'item46' },
                        { title: 'Title Text', key: 'item47' },
                        { title: 'Title Text', key: 'item48' },
                        { title: 'Title Text', key: 'item49' },
                        { title: 'Title Text', key: 'item50' },
                    ]}
                    refreshing={false}
                    onRefresh={async () => {await 15;}}
                    renderItem={({ item, separators }) => (
                        <TouchableWithoutFeedback
                            key={item.key}
                            onPress={() => {}}
                            onShowUnderlay={separators.highlight}
                            onHideUnderlay={separators.unhighlight}>
                            <View style={{ backgroundColor: 'white', borderColor:'blakc', borderWidth:1, borderRadius:3, alignItems: 'center', marginVertical:5 }}>
                                <Text>{item.title}</Text>
                                <Text>{item.key}</Text>
                            </View>
                        </TouchableWithoutFeedback>
                    )}
                    style={{ backgroundColor: 'red', flex: 1 }}
                    /> */}
                    <MyChannel />
                </View>
            </TouchableWithoutFeedback>
            <View style={{backgroundColor: 'green'}}>
                <InputBar />
                <SafeAreaView />
            </View>
        </KeyboardAvoidingView>
    );
}

export default HomeScreen;
