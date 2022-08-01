import { InputType, Field } from '@nestjs/graphql';
import { IsString, IsNotEmpty } from 'class-validator';
import { User } from '../users/interfaces/user.entity';

@InputType()
export class MessageInput {
    @Field(() => User, { description: 'sender of the message' })
    @IsNotEmpty()
    readonly sender: User;
    @Field(() => String, { description: 'Content of the message' })
    @IsString()
    @IsNotEmpty()
    readonly content: string;
}
