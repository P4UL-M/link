import { ObjectType, Field, registerEnumType } from '@nestjs/graphql';
import { IsNotEmpty } from 'class-validator';
import { Message } from './message.entity';

export enum Action {
    NEW,
    UPDATE,
    DELETE,
}

registerEnumType(Action, {
    name: 'Action',
    description: 'Action of the message',
});

@ObjectType()
export class EventMessage {
    @Field(() => Message, { description: 'message of the event' })
    @IsNotEmpty()
    readonly message: Message;

    @Field(() => Action, { description: 'type of the event' })
    @IsNotEmpty()
    readonly type: Action;
}
