import { Column, Entity, PrimaryGeneratedColumn } from 'typeorm';
import { ObjectType, Field, ID } from '@nestjs/graphql';
import { IsEmail, IsString, IsNotEmpty } from 'class-validator';
import { pwdMiddleware, idMiddleware, credidentialMiddleware } from '../../auth/auth.middleware';

@Entity('users')
@ObjectType()
export class User {
    @Field(() => ID, { middleware: [idMiddleware] })
    @IsString()
    @IsNotEmpty()
    @PrimaryGeneratedColumn('uuid')
    readonly _id?: string;

    @Field(() => String, { description: 'Pseudo of the user' })
    @IsString()
    @IsNotEmpty()
    @Column()
    readonly pseudo: string;

    @Field(() => String, { description: 'E-mail of the user' })
    @IsEmail()
    @IsNotEmpty()
    @Column()
    email: string;

    @Field(() => String, { description: 'Password of the user', middleware: [pwdMiddleware] })
    @IsString()
    @IsNotEmpty()
    @Column()
    password: string;

    @Field(() => String, { description: 'Public key of the user' })
    @IsString()
    @IsNotEmpty()
    @Column()
    publicKey: string;

    @Field(() => Number, {
        description: 'access of the user : bin rwrw',
        middleware: [credidentialMiddleware],
    })
    @IsNotEmpty()
    @Column()
    credidential: number;
}
