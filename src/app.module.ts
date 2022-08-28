import { Module, UnauthorizedException } from '@nestjs/common';
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
import { AuthService } from './auth/auth.service';

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
        // GraphQLModule.forRoot<ApolloDriverConfig>({
        //     driver: ApolloDriver,
        //     autoSchemaFile: 'schema.gql',
        //     transformSchema: (schema) => regexDirectiveTransformer(schema, 'constraint'),
        //     buildSchemaOptions: {
        //         directives: [
        //             new GraphQLDirective({
        //                 name: 'constraint',
        //                 args: {
        //                     pattern: {
        //                         type: GraphQLString,
        //                     },
        //                 },
        //                 locations: [DirectiveLocation.FIELD_DEFINITION],
        //             }),
        //         ],
        //     },
        //     subscriptions: {
        //         'graphql-ws': {
        //             onConnect: (connectionParams, webSocket, context) => {},
        //             onDisconnect: (webSocket, context) => {},
        //         },
        //         'subscriptions-transport-ws': true,
        //     },
        //     plugins: [setHttpPlugin, ...protection.plugins],
        //     validationRules: [...protection.validationRules],
        // }),
        GraphQLModule.forRootAsync({
            driver: ApolloDriver,
            imports: [AuthModule],
            useFactory: async (authService: AuthService) => ({
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
                    'graphql-ws': {
                        onConnect: async (ctx) => {
                            const { connectionParams } = ctx;
                            const token = connectionParams.Authorization.split(' ')[1];
                            if (token) {
                                const user = await authService.verifyToken(token);
                                if (user) {
                                    ctx.extra.req = {
                                        user: {
                                            ...user,
                                        },
                                    };
                                }
                            }
                        },
                        onDisconnect: (webSocket, context) => {
                            null;
                        },
                    },
                    'subscriptions-transport-ws': {
                        onConnect: async (connectionParams: any, webSocket, context) => {
                            const token = connectionParams.Authorization.split(' ')[1];
                            if (token) {
                                const user = await authService.verifyToken(token);
                                if (user) {
                                    context.req = {
                                        user: {
                                            ...user,
                                        },
                                    };
                                }
                            }
                            return context;
                        },
                    },
                },
                context: async ({ extra }) => {
                    return extra;
                },
                plugins: [setHttpPlugin, ...protection.plugins],
                validationRules: [...protection.validationRules],
            }),
            // inject: AuthService
            inject: [AuthService],
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
