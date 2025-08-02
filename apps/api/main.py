from fastapi import FastAPI, HTTPException, UploadFile, File
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Dict, Any, Optional
import uvicorn
from PIL import Image
import base64
import io
import os
from pathlib import Path
import google.generativeai as genai
import fal_client
from datetime import datetime
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize FastAPI app
app = FastAPI(
    title="GroupWeave AI Backend",
    description="AI-powered backend with Gemini vision and Fal.ai generation",
    version="1.0.0"
)

# Configure Gemini API
genai.configure(api_key=os.getenv("GEMINI_API_KEY"))
gemini_model = genai.GenerativeModel('gemini-pro-vision')

# Configure Fal.ai client
fal_client.api_key = os.getenv("FAL_API_KEY")

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Pydantic models
class VotingAnalysisRequest(BaseModel):
    poll_id: int
    votes: List[int]
    options: List[str]
    metadata: Optional[Dict[str, Any]] = None

class VotingAnalysisResponse(BaseModel):
    poll_id: int
    winner_index: int
    winner_option: str
    confidence_score: float
    analysis: Dict[str, Any]
    recommendations: List[str]

class GovernanceInsight(BaseModel):
    insight_type: str
    title: str
    description: str
    confidence: float
    data: Dict[str, Any]
    timestamp: datetime

class StakingOptimization(BaseModel):
    user_address: str
    current_stake: float
    recommended_action: str
    potential_rewards: float
    risk_assessment: str
    reasoning: str

class AIDecisionRequest(BaseModel):
    context: str
    options: List[str]
    criteria: List[str]
    weights: Optional[List[float]] = None

class AIDecisionResponse(BaseModel):
    recommended_option: str
    confidence: float
    reasoning: str
    analysis: Dict[str, Any]

class ImageAnalysisRequest(BaseModel):
    image_data: str  # Base64 encoded image
    prompt: Optional[str] = "Analyze this image in the context of governance and decision making"

class ImageAnalysisResponse(BaseModel):
    analysis: str
    insights: List[str]
    governance_relevance: str
    confidence: float
    metadata: Dict[str, Any]

class ImageGenerationRequest(BaseModel):
    prompt: str
    style: Optional[str] = "realistic"
    aspect_ratio: Optional[str] = "1:1"
    num_images: Optional[int] = 1

class ImageGenerationResponse(BaseModel):
    images: List[str]  # URLs or base64 encoded images
    prompt_used: str
    generation_time: float
    metadata: Dict[str, Any]

class VideoGenerationRequest(BaseModel):
    prompt: str
    duration: Optional[int] = 5  # seconds
    fps: Optional[int] = 24
    style: Optional[str] = "realistic"

class VideoGenerationResponse(BaseModel):
    video_url: str
    thumbnail_url: Optional[str]
    prompt_used: str
    generation_time: float
    metadata: Dict[str, Any]

# In-memory storage (replace with proper database in production)
voting_data = {}
governance_insights = []
staking_data = {}

@app.get("/")
async def root():
    return {
        "message": "GroupWeave AI Backend",
        "version": "1.0.0",
        "status": "running",
        "timestamp": datetime.now().isoformat()
    }

@app.get("/health")
async def health_check():
    return {"status": "healthy", "timestamp": datetime.now().isoformat()}

@app.post("/ai/analyze-voting", response_model=VotingAnalysisResponse)
async def analyze_voting(request: VotingAnalysisRequest):
    """
    Analyze voting results and determine winner with AI insights
    """
    try:
        # Simple voting analysis (replace with sophisticated AI model)
        total_votes = sum(request.votes)
        if total_votes == 0:
            raise HTTPException(status_code=400, detail="No votes to analyze")
        
        # Find winner
        winner_index = request.votes.index(max(request.votes))
        winner_option = request.options[winner_index]
        winner_votes = request.votes[winner_index]
        
        # Calculate confidence score
        confidence_score = winner_votes / total_votes
        
        # Generate analysis
        analysis = {
            "total_votes": total_votes,
            "vote_distribution": dict(zip(request.options, request.votes)),
            "margin_of_victory": winner_votes - sorted(request.votes, reverse=True)[1] if len(request.votes) > 1 else winner_votes,
            "participation_rate": "high" if total_votes > 100 else "medium" if total_votes > 50 else "low"
        }
        
        # Generate recommendations
        recommendations = []
        if confidence_score < 0.6:
            recommendations.append("Consider extending voting period due to close results")
        if total_votes < 50:
            recommendations.append("Low participation - consider incentivizing voter engagement")
        if max(request.votes) == min(request.votes):
            recommendations.append("Tied vote - may need additional decision criteria")
        
        # Store results
        voting_data[request.poll_id] = {
            "analysis": analysis,
            "winner": winner_option,
            "timestamp": datetime.now().isoformat()
        }
        
        return VotingAnalysisResponse(
            poll_id=request.poll_id,
            winner_index=winner_index,
            winner_option=winner_option,
            confidence_score=confidence_score,
            analysis=analysis,
            recommendations=recommendations
        )
    
    except Exception as e:
        logger.error(f"Error analyzing voting: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Analysis failed: {str(e)}")

@app.post("/ai/governance-insights", response_model=List[GovernanceInsight])
async def generate_governance_insights(data: Dict[str, Any]):
    """
    Generate AI-powered governance insights
    """
    try:
        insights = []
        
        # Example insights (replace with actual AI analysis)
        if "voting_patterns" in data:
            insights.append(GovernanceInsight(
                insight_type="voting_pattern",
                title="Voting Participation Trend",
                description="Voter participation has increased by 15% over the last month",
                confidence=0.85,
                data={"trend": "increasing", "percentage_change": 15},
                timestamp=datetime.now()
            ))
        
        if "staking_data" in data:
            insights.append(GovernanceInsight(
                insight_type="staking_analysis",
                title="Staking Concentration",
                description="Top 10% of stakers control 60% of voting power",
                confidence=0.92,
                data={"concentration_ratio": 0.6, "top_percentile": 0.1},
                timestamp=datetime.now()
            ))
        
        # Store insights
        governance_insights.extend(insights)
        
        return insights
    
    except Exception as e:
        logger.error(f"Error generating insights: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Insight generation failed: {str(e)}")

@app.post("/ai/staking-optimization", response_model=StakingOptimization)
async def optimize_staking(user_address: str, current_stake: float, market_data: Dict[str, Any]):
    """
    Provide AI-powered staking optimization recommendations
    """
    try:
        # Simple optimization logic (replace with sophisticated AI model)
        if current_stake < 100:
            recommended_action = "increase_stake"
            potential_rewards = current_stake * 0.12  # 12% APY
            risk_assessment = "low"
            reasoning = "Low stake amount - increasing stake will improve rewards significantly"
        elif current_stake > 10000:
            recommended_action = "diversify"
            potential_rewards = current_stake * 0.08  # 8% APY
            risk_assessment = "medium"
            reasoning = "High stake concentration - consider diversifying across multiple validators"
        else:
            recommended_action = "maintain"
            potential_rewards = current_stake * 0.10  # 10% APY
            risk_assessment = "low"
            reasoning = "Optimal stake amount - maintain current position"
        
        optimization = StakingOptimization(
            user_address=user_address,
            current_stake=current_stake,
            recommended_action=recommended_action,
            potential_rewards=potential_rewards,
            risk_assessment=risk_assessment,
            reasoning=reasoning
        )
        
        # Store optimization data
        staking_data[user_address] = {
            "optimization": optimization.dict(),
            "timestamp": datetime.now().isoformat()
        }
        
        return optimization
    
    except Exception as e:
        logger.error(f"Error optimizing staking: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Optimization failed: {str(e)}")

@app.post("/ai/decision-support", response_model=AIDecisionResponse)
async def ai_decision_support(request: AIDecisionRequest):
    """
    Provide AI-powered decision support for governance proposals
    """
    try:
        # Simple decision logic (replace with sophisticated AI model)
        weights = request.weights or [1.0] * len(request.criteria)
        
        # Score each option (placeholder logic)
        option_scores = {}
        for i, option in enumerate(request.options):
            # Simple scoring based on option index and criteria
            score = sum(w * (0.5 + 0.1 * (i + j)) for j, w in enumerate(weights))
            option_scores[option] = min(score, 1.0)  # Cap at 1.0
        
        # Find best option
        recommended_option = max(option_scores, key=option_scores.get)
        confidence = option_scores[recommended_option]
        
        reasoning = f"Based on the provided criteria and context, {recommended_option} scores highest with a confidence of {confidence:.2f}"
        
        analysis = {
            "option_scores": option_scores,
            "criteria_weights": dict(zip(request.criteria, weights)),
            "context_analysis": "Analyzed based on provided context and criteria"
        }
        
        return AIDecisionResponse(
            recommended_option=recommended_option,
            confidence=confidence,
            reasoning=reasoning,
            analysis=analysis
        )
    
    except Exception as e:
        logger.error(f"Error in decision support: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Decision support failed: {str(e)}")

@app.get("/data/voting-history")
async def get_voting_history():
    """Get historical voting data"""
    return voting_data

@app.get("/data/governance-insights")
async def get_governance_insights():
    """Get stored governance insights"""
    return governance_insights

@app.get("/data/staking-data")
async def get_staking_data():
    """Get staking optimization data"""
    return staking_data

@app.post("/ai/analyze-image", response_model=ImageAnalysisResponse)
async def analyze_image_with_gemini(request: ImageAnalysisRequest):
    """
    Analyze images using Google Gemini Vision for governance insights
    """
    try:
        # Decode base64 image
        image_data = base64.b64decode(request.image_data)
        image = Image.open(io.BytesIO(image_data))
        
        # Prepare prompt for governance context
        governance_prompt = f"""
        {request.prompt}
        
        Please analyze this image and provide:
        1. A detailed description of what you see
        2. Any governance or decision-making relevance
        3. Potential insights for community voting or proposals
        4. Risk assessment if applicable
        5. Recommendations for action
        
        Focus on aspects relevant to decentralized governance, community decision-making, and organizational transparency.
        """
        
        # Generate content with Gemini
        response = gemini_model.generate_content([governance_prompt, image])
        analysis_text = response.text
        
        # Parse insights (simple extraction - could be enhanced with NLP)
        insights = []
        lines = analysis_text.split('\n')
        for line in lines:
            if any(keyword in line.lower() for keyword in ['insight:', 'recommendation:', 'risk:', 'opportunity:']):
                insights.append(line.strip())
        
        # Determine governance relevance
        governance_keywords = ['vote', 'proposal', 'community', 'decision', 'governance', 'transparency', 'accountability']
        relevance_score = sum(1 for keyword in governance_keywords if keyword in analysis_text.lower())
        governance_relevance = "high" if relevance_score >= 3 else "medium" if relevance_score >= 1 else "low"
        
        # Calculate confidence based on response length and keyword presence
        confidence = min(0.95, 0.5 + (len(analysis_text) / 1000) + (relevance_score * 0.1))
        
        return ImageAnalysisResponse(
            analysis=analysis_text,
            insights=insights[:5],  # Limit to top 5 insights
            governance_relevance=governance_relevance,
            confidence=confidence,
            metadata={
                "image_size": f"{image.width}x{image.height}",
                "image_format": image.format,
                "analysis_length": len(analysis_text),
                "keyword_matches": relevance_score,
                "timestamp": datetime.now().isoformat()
            }
        )
    
    except Exception as e:
        logger.error(f"Error analyzing image with Gemini: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Image analysis failed: {str(e)}")

@app.post("/ai/generate-image", response_model=ImageGenerationResponse)
async def generate_image_with_fal(request: ImageGenerationRequest):
    """
    Generate images using Fal.ai for governance visualizations
    """
    try:
        start_time = datetime.now()
        
        # Enhance prompt for governance context
        enhanced_prompt = f"""
        {request.prompt}
        
        Style: {request.style}, professional, clean, suitable for governance and community presentations.
        High quality, detailed, appropriate for decentralized organization materials.
        """
        
        # Call Fal.ai API for image generation
        handler = await fal_client.submit_async(
            "fal-ai/flux/schnell",
            arguments={
                "prompt": enhanced_prompt,
                "image_size": request.aspect_ratio,
                "num_images": request.num_images,
                "enable_safety_checker": True
            }
        )
        
        result = await handler.get()
        
        # Extract image URLs
        images = []
        if "images" in result:
            images = [img["url"] for img in result["images"]]
        elif "image" in result:
            images = [result["image"]["url"]]
        
        generation_time = (datetime.now() - start_time).total_seconds()
        
        return ImageGenerationResponse(
            images=images,
            prompt_used=enhanced_prompt,
            generation_time=generation_time,
            metadata={
                "model": "fal-ai/flux/schnell",
                "style": request.style,
                "aspect_ratio": request.aspect_ratio,
                "num_images": len(images),
                "timestamp": datetime.now().isoformat()
            }
        )
    
    except Exception as e:
        logger.error(f"Error generating image with Fal.ai: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Image generation failed: {str(e)}")

@app.post("/ai/generate-video", response_model=VideoGenerationResponse)
async def generate_video_with_fal(request: VideoGenerationRequest):
    """
    Generate videos using Fal.ai for governance presentations
    """
    try:
        start_time = datetime.now()
        
        # Enhance prompt for governance context
        enhanced_prompt = f"""
        {request.prompt}
        
        Style: {request.style}, professional, suitable for governance presentations and community communications.
        Duration: {request.duration} seconds. Clean, engaging, appropriate for decentralized organization content.
        """
        
        # Call Fal.ai API for video generation
        handler = await fal_client.submit_async(
            "fal-ai/runway-gen3/turbo/image-to-video",
            arguments={
                "prompt": enhanced_prompt,
                "duration": request.duration,
                "fps": request.fps,
                "motion_bucket_id": 127,
                "cond_aug": 0.02
            }
        )
        
        result = await handler.get()
        
        video_url = result.get("video", {}).get("url", "")
        thumbnail_url = result.get("thumbnail", {}).get("url")
        
        generation_time = (datetime.now() - start_time).total_seconds()
        
        return VideoGenerationResponse(
            video_url=video_url,
            thumbnail_url=thumbnail_url,
            prompt_used=enhanced_prompt,
            generation_time=generation_time,
            metadata={
                "model": "fal-ai/runway-gen3/turbo",
                "duration": request.duration,
                "fps": request.fps,
                "style": request.style,
                "timestamp": datetime.now().isoformat()
            }
        )
    
    except Exception as e:
        logger.error(f"Error generating video with Fal.ai: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Video generation failed: {str(e)}")

@app.post("/ai/analyze-image-upload")
async def analyze_uploaded_image(file: UploadFile = File(...), prompt: Optional[str] = None):
    """
    Analyze uploaded image files using Gemini Vision
    """
    try:
        # Read and validate image file
        if not file.content_type.startswith('image/'):
            raise HTTPException(status_code=400, detail="File must be an image")
        
        contents = await file.read()
        image_b64 = base64.b64encode(contents).decode('utf-8')
        
        # Use the existing image analysis endpoint
        request = ImageAnalysisRequest(
            image_data=image_b64,
            prompt=prompt or "Analyze this image in the context of governance and decision making"
        )
        
        return await analyze_image_with_gemini(request)
    
    except Exception as e:
        logger.error(f"Error analyzing uploaded image: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Image upload analysis failed: {str(e)}")

if __name__ == "__main__":
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=8000,
        reload=True,
        log_level="info"
    )