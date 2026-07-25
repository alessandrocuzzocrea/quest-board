// Generated types matching the Rust backend
export interface User {
  id: string;
  username: string;
  role: string;
  created_at: string;
  updated_at: string;
}

export interface UserResponse {
  id: string;
  username: string;
  role: string;
}

export interface Board {
  id: string;
  name: string;
  position: number;
  slug: string;
  created_by: string;
  created_at: string;
  updated_at: string;
}

export interface ListWithCards {
  id: string;
  board_id: string;
  name: string | null;
  position: number;
  type: string;
  color: string | null;
  cards: CardWithMembers[];
  created_at: string;
  updated_at: string;
}

export interface CardWithMembers {
  id: string;
  board_id: string;
  list_id: string;
  position: number;
  name: string;
  description: string | null;
  start_date: string | null;
  due_date: string | null;
  is_due_completed: boolean;
  is_closed: boolean;
  created_by: string;
  members: UserResponse[];
  labels: Label[];
  comments_count: bigint;
  checklists: TaskListWithTasks[];
  created_at: string;
  updated_at: string;
}

export interface Label {
  id: string;
  board_id: string;
  name: string;
  color: string;
  position: number;
  created_at: string;
  updated_at: string;
}

export interface CommentWithUser {
  id: string;
  card_id: string;
  user_id: string;
  user: UserResponse | null;
  text: string;
  created_at: string;
  updated_at: string;
}

export interface Action {
  id: string;
  card_id: string;
  board_id: string | null;
  user_id: string | null;
  type: string;
  data: string;
  created_at: string;
}

export interface Attachment {
  id: string;
  card_id: string;
  user_id: string;
  name: string;
  type: string;
  file_path: string | null;
  link_url: string | null;
  size: bigint | null;
  mime_type: string | null;
  created_at: string;
}

export interface TaskListWithTasks {
  id: string;
  card_id: string;
  name: string;
  position: number;
  hide_completed: boolean;
  tasks: Task[];
  created_at: string;
  updated_at: string;
}

export interface Task {
  id: string;
  task_list_id: string;
  name: string;
  position: number;
  is_done: boolean;
  created_at: string;
  updated_at: string;
}

export interface FullCard {
  id: string;
  board_id: string;
  list_id: string;
  position: number;
  name: string;
  description: string | null;
  start_date: string | null;
  due_date: string | null;
  is_due_completed: boolean;
  is_closed: boolean;
  created_by: string;
  members: UserResponse[];
  labels: Label[];
  comments: CommentWithUser[];
  actions: Action[];
  attachments: Attachment[];
  checklists: TaskListWithTasks[];
  comments_count: bigint;
  created_at: string;
  updated_at: string;
}

export interface BoardResponse {
  board: Board;
  lists: ListWithCards[];
}

export interface SSEEvent {
  board_id?: string;
  card_id?: string;
  list_id?: string;
}
