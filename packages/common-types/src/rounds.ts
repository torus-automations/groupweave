export interface Image {
  id: number;
  data: string; // Base64 encoded image
  votes: number;
}

export interface Round {
  id: number;
  criteria: string;
  images: Image[];
}

export interface CreateRoundRequest {
  criteria: string;
  image1_data: string; // Base64 encoded image
  image2_data: string; // Base64 encoded image
}

export interface VoteRequest {
  image_id: number;
}