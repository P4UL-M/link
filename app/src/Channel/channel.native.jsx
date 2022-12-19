import React, { useEffect, useContext, useRef } from 'react';
import {
    View,
    Text,
    FlatList,
    ActivityIndicator,
    TouchableOpacity
} from 'react-native';
import { gql, useQuery } from '@apollo/client';
import { AuthContext } from '../context/AuthContext';

const SUB = gql`subscription {
        ChannelMessage {
            message {
                _id
                content
                sender {
                    _id
                    pseudo
                }
                date
            }
        type
        }
    }
`;
const MESSAGES = gql`query {
    Messages {
        _id
        content
        sender {
            _id
            pseudo
        }
        date
    }
}`;

function MyChannel({user}) {
    // ref to the flatlist
    const flatListRef = useRef(null);

    const authContext = useContext(AuthContext);

    const { data, subscribeToMore } = useQuery(MESSAGES, {
        context: {
            headers: {
                'Authorization': `Bearer ${authContext.getAccessToken()}`,
            },
        },
    });

    useEffect(() => {
        subscribeToMore({
            document: SUB,
            shouldResubscribe: true,
            updateQuery: (prev, { subscriptionData }) => {
                if (!subscriptionData?.data?.ChannelMessage) return prev;
                const newMessage = subscriptionData.data.ChannelMessage;
                if (newMessage?.type === 'NEW') {
                    return Object.assign({}, prev, {
                        Messages: [...prev.Messages, newMessage.message],
                    });
                }
            },
            context: {
                headers: {
                    'Authorization': `Bearer ${authContext.getAccessToken()}`,
                },
            },
        });
    }, [subscribeToMore]);

    return (
        <FlatList
            data={data?.Messages || []}
            renderItem={({item}) => (
                <TouchableOpacity style={{paddingLeft:15, paddingBottom:5, alignItems: item?.sender.pseudo == user?.pseudo && item?.sender.publicKey == user?.publicKey ? 'flex-start' : 'flex-end'}}>
                    <Text>{item?.sender.pseudo}</Text>
                    <Text>{item?.content}</Text>
                </TouchableOpacity>
            )}
            keyExtractor={item => item._id}
            ListEmptyComponent={() => (
                <View>
                    <ActivityIndicator size="large" color="#0000ff" />
                </View>
            )}
            extraData={data?.Messages || []}
            style={{flexGrow: 1, flex:1}}
            ref={flatListRef}
            onLayout={() => {
                if (flatListRef.current && data?.Messages) {
                    flatListRef.current.scrollToEnd({animated: true});
                }
            } }
            onContentSizeChange={() => {
                if (flatListRef.current && data?.Messages) {
                    flatListRef.current.scrollToEnd({animated: true});
                }
            } }
        />
    );
}

export default MyChannel;