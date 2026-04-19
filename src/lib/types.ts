export interface User    { id: any; username: string; email?: string; avatar?: string; created?: string; }
export interface Room    { id: any; name?: string; kind?: 'public' | 'private' | 'direct'; direct_key?: string; created: string; updated?: string; created_by?: any; last_message?: Message; unread_count?: number; other_user?: User; }
export interface RoomMember { id: any; room: any; user: any; role: 'owner' | 'member'; joined: string; last_read_at?: string; muted?: boolean; }
export interface MessageReactionSummary { emoji: string; count: number; reacted_by_me: boolean; }
export interface Message { id: any; room: any; author: any; author_username?: string; body: string; created: string; updated?: string; deleted?: boolean; reply_to?: any; reactions?: MessageReactionSummary[]; }
export interface UserSearchResult { id: any; username: string; avatar?: string; }
export interface LiveEvent { action: 'Create' | 'Update' | 'Delete'; data: Message; }
export interface ContextMenuItem { label: string; action: () => void; }
