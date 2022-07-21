import { ObjectType, Field, ID } from '@nestjs/graphql';
import { IsEmail, IsString, IsNotEmpty, IsDate, IsOptional } from 'class-validator';

@ObjectType()
export class UserType {
    @Field(() => ID)
    @IsString()
    @IsNotEmpty()
    readonly id?: string;
    @Field(() => String, { description: 'Pseudo of the user' })
    @IsString()
    @IsNotEmpty()
    readonly pseudo: string;
    @Field(() => String, { description: 'E-mail of the user' })
    @IsEmail()
    @IsNotEmpty()
    readonly email: string;
    @Field(() => String, { description: 'Password of the user' })
    @IsString()
    @IsNotEmpty()
    readonly password: string;
    @Field(() => String, { description: 'Public key of the user' })
    @IsString()
    @IsNotEmpty()
    readonly publicKey: string;
}
