import { Module } from '@nestjs/common';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { GraphQLModule } from '@nestjs/graphql';
import { TypeOrmModule } from '@nestjs/typeorm';
import { UsersModule } from './users/users.module';
import { MessagesModule } from './messages/messages.module';
import { ApolloDriver, ApolloDriverConfig } from '@nestjs/apollo';
import { AuthModule } from './auth/auth.module';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { configuration } from '../config/configuration';
import { validationSchema } from '../config/validation';
import { setHttpPlugin } from './auth/graphQL.plugin';
import { ApolloArmor } from '@escape.tech/graphql-armor';
import { regexDirectiveTransformer } from './directives/constraints.graphql';
import { DirectiveLocation, GraphQLDirective, GraphQLString } from 'graphql';
import { PubSubModule } from './pubsub/pubsub.module';

const armor = new ApolloArmor();
const protection = armor.protect();

@Module({
    imports: [
        ConfigModule.forRoot({
            envFilePath: `${process.cwd()}/config/env/.env`,
            load: [configuration],
            isGlobal: true,
            validationSchema,
        }),
        GraphQLModule.forRoot<ApolloDriverConfig>({
            driver: ApolloDriver,
            autoSchemaFile: 'schema.gql',
            transformSchema: (schema) => regexDirectiveTransformer(schema, 'constraint'),
            buildSchemaOptions: {
                directives: [
                    new GraphQLDirective({
                        name: 'constraint',
                        args: {
                            pattern: {
                                type: GraphQLString,
                            },
                        },
                        locations: [DirectiveLocation.FIELD_DEFINITION],
                    }),
                ],
            },
            subscriptions: {
                'graphql-ws': true,
                'subscriptions-transport-ws': true,
            },
            plugins: [setHttpPlugin, ...protection.plugins],
            validationRules: [...protection.validationRules],
        }),
        TypeOrmModule.forRootAsync({
            imports: [ConfigModule],
            useFactory: (configService: ConfigService) => ({
                type: 'mysql',
                host: '127.0.0.1',
                port: 3306,
                username: configService.get('database.username'),
                password: configService.get('database.password'),
                database: 'link',
                entities: ['dist/**/*.entity.js'],
                synchronize: true,
            }),
            inject: [ConfigService],
        }),
        UsersModule,
        AuthModule,
        MessagesModule,
    ],
    controllers: [AppController],
    providers: [AppService],
})
export class AppModule {}
