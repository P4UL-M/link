import React from 'react';
import stylesheet from './style';
import {
    View,
    Text,
    FlatList,
} from 'react-native';
import { useTheme } from '@react-navigation/native';
import { useSubscription, gql } from '@apollo/client';

const SUB = gql`
    subscription {
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

function MyChannel() {
    // const data = [
    //     {
    //         id: '1',
    //         author: {
    //             pseuod: 'John Doe',
    //             id: '1',
    //         },
    //         content: 'Hello World',
    //         date: '2020-01-01',
    //     },
    // ]
    const { colors } = useTheme();
    const styles = stylesheet(colors);

    const {data} = useSubscription(SUB);
    console.log(data);

    return (
        <View style={styles.container}>
            <FlatList
                data={data?.ChannelMessage}
                renderItem={({item}) => (
                    <View>
                        <Text>{item.message.sender.pseudo}</Text>
                        <Text>{item.message.content}</Text>
                    </View>
                )}
                keyExtractor={item => item.message._id}
            />
        </View>
    );
}

export default MyChannel;