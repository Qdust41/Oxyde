export interface User    { id: any; username: string; email: string; avatar?: string; created: string; }
export interface Room    { id: any; name: string; created: string; }
export interface Message { id: any; room: any; author: any; author_username?: string; body: string; created: string; }
export interface LiveEvent { action: 'Create' | 'Update' | 'Delete'; data: Message; }
export interface ContextMenuItem { label: string; action: () => void; }
