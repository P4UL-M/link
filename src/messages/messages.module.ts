import { Module } from '@nestjs/common';
import { MessagesResolver } from './messages.resolver';
import { MessagesService } from './messages.service';
import { TypeOrmModule } from '@nestjs/typeorm';
import { Message } from './interfaces/message.entity';

@Module({
    imports: [TypeOrmModule.forFeature([Message])],
    controllers: [],
    providers: [MessagesResolver, MessagesService],
    exports: [MessagesService],
})
export class MessagesModule {}
