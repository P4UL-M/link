import { Module } from '@nestjs/common';
import { MessagesResolver } from './messages.resolver';
import { MessagesService } from './messages.service';
import { TypeOrmModule } from '@nestjs/typeorm';
import { Message } from './interfaces/message.entity';
import { UsersModule } from '../users/users.module';
import { PubSubModule } from 'src/pubsub/pubsub.module';

@Module({
    imports: [TypeOrmModule.forFeature([Message]), UsersModule, PubSubModule],
    controllers: [],
    providers: [MessagesResolver, MessagesService],
    exports: [MessagesService],
})
export class MessagesModule {}
