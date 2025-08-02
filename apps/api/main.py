from fastapi import FastAPI, HTTPException, Depends
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Optional, Dict, Any
import uvicorn
import logging
from datetime import datetime
import asyncio
import json

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize FastAPI app
app = FastAPI(
    title="GroupWeave AI Backend",
    description="AI-powered backend for decentralized governance and decision making",
    version="1.0.0"
)

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

if __name__ == "__main__":
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=8000,
        reload=True,
        log_level="info"
    )