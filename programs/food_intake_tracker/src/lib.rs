use anchor_lang::prelude::*;

declare_id!("A9uDkyTXNtud3R5SqXAETBysrWxc1z9hPSgcw8DbZ7TC");

#[program]
pub mod food_intake_tracker {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let tracker = &mut ctx.accounts.tracker;
        tracker.owner = ctx.accounts.user.key();
        tracker.current_streak = 0;
        tracker.longest_streak = 0;
        tracker.total_points = 0;
        tracker.last_meal_date = 0;
        tracker.today_meals = MealStatus::default();
        tracker.bump = ctx.bumps.tracker;

        msg!("Food tracker initialized for user: {}", tracker.owner);
        Ok(())
    }

    // Log a meal (breakfast, lunch, or dinner)
    pub fn log_meal(ctx: Context<LogMeal>, meal_type: MealType) -> Result<()> {
        let tracker = &mut ctx.accounts.tracker;
        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;

        // Get current date (days since epoch)
        let current_day = current_timestamp / 86400; // 86400 seconds in a day
        let last_day = tracker.last_meal_date / 86400;

        // Check if it's a new day
        if current_day > last_day {
            // Check if previous day completed all meals for streak
            if last_day > 0 {
                if tracker.today_meals.is_complete() && (current_day - last_day == 1) {
                    // Consecutive day with all meals - increment streak
                    tracker.current_streak += 1;

                    // Check for streak milestones
                    match tracker.current_streak {
                        7 => {
                            tracker.total_points += 100;
                            msg!("🔥 7-day streak! Bonus 100 points!");
                        }
                        30 => {
                            tracker.total_points += 500;
                            msg!("🔥 30-day streak! Bonus 500 points!");
                        }
                        100 => {
                            tracker.total_points += 2000;
                            msg!("🔥 100-day streak! Bonus 2000 points!");
                        }
                        _ => {}
                    }

                    // Update longest streak
                    if tracker.current_streak > tracker.longest_streak {
                        tracker.longest_streak = tracker.current_streak;
                    }
                } else if current_day - last_day > 1 || !tracker.today_meals.is_complete() {
                    // Missed a day or didn't complete previous day - reset streak
                    tracker.current_streak = 0;
                    msg!("Streak reset. Keep going!");
                }
            }

            // Reset today's meals for new day
            tracker.today_meals = MealStatus::default();
        }

        // Validate meal time window
        require!(
            validate_meal_time(current_timestamp, &meal_type),
            FoodTrackerError::InvalidMealTime
        );

        // Check if meal already logged
        let already_logged = match meal_type {
            MealType::Breakfast => tracker.today_meals.breakfast,
            MealType::Lunch => tracker.today_meals.lunch,
            MealType::Dinner => tracker.today_meals.dinner,
        };

        require!(!already_logged, FoodTrackerError::MealAlreadyLogged);

        // Log the meal
        match meal_type {
            MealType::Breakfast => {
                tracker.today_meals.breakfast = true;
                tracker.today_meals.timestamp_breakfast = Some(current_timestamp);
                msg!("🍳 Breakfast logged!");
            }
            MealType::Lunch => {
                tracker.today_meals.lunch = true;
                tracker.today_meals.timestamp_lunch = Some(current_timestamp);
                msg!("🍱 Lunch logged!");
            }
            MealType::Dinner => {
                tracker.today_meals.dinner = true;
                tracker.today_meals.timestamp_dinner = Some(current_timestamp);
                msg!("🍽️ Dinner logged!");
            }
        }

        // Award points for meal
        tracker.total_points += 10;
        tracker.last_meal_date = current_timestamp;

        // Check if all 3 meals completed for bonus
        if tracker.today_meals.is_complete() {
            tracker.total_points += 50; // Bonus for completing all meals
            msg!("✨ All 3 meals completed today! +50 bonus points!");
        }

        msg!(
            "Total points: {} | Current streak: {} days",
            tracker.total_points,
            tracker.current_streak
        );

        Ok(())
    }

    // Get user stats (view function)
    pub fn get_stats(ctx: Context<GetStats>) -> Result<()> {
        let tracker = &ctx.accounts.tracker;

        msg!("=== Food Tracker Stats ===");
        msg!("Total Points: {}", tracker.total_points);
        msg!("Current Streak: {} days", tracker.current_streak);
        msg!("Longest Streak: {} days", tracker.longest_streak);
        msg!("Today's Meals:");
        msg!(
            "  Breakfast: {}",
            if tracker.today_meals.breakfast {
                "✅"
            } else {
                "❌"
            }
        );
        msg!(
            "  Lunch: {}",
            if tracker.today_meals.lunch {
                "✅"
            } else {
                "❌"
            }
        );
        msg!(
            "  Dinner: {}",
            if tracker.today_meals.dinner {
                "✅"
            } else {
                "❌"
            }
        );

        Ok(())
    }
}

fn validate_meal_time(timestamp: i64, meal_type: &MealType) -> bool {
    // Get hour of day (0-23) in UTC
    let seconds_in_day = timestamp % 86400;
    let hour = (seconds_in_day / 3600) as u8;

    match meal_type {
        MealType::Breakfast => hour >= 5 && hour < 11, // 5 AM - 11 AM
        MealType::Lunch => hour >= 11 && hour < 16,    // 11 AM - 4 PM
        MealType::Dinner => hour >= 16 && hour < 23,   // 4 PM - 11 PM
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + FoodTrackerAccount::INIT_SPACE,
        seeds = [b"food_tracker", user.key().as_ref()],
        bump
    )]
    pub tracker: Account<'info, FoodTrackerAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LogMeal<'info> {
    #[account(
        mut,
        seeds = [b"food_tracker", user.key().as_ref()],
        bump = tracker.bump,
        constraint = tracker.owner == user.key() @ FoodTrackerError::Unauthorized
    )]
    pub tracker: Account<'info, FoodTrackerAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetStats<'info> {
    #[account(
        seeds = [b"food_tracker", user.key().as_ref()],
        bump = tracker.bump,
    )]
    pub tracker: Account<'info, FoodTrackerAccount>,

    pub user: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct FoodTrackerAccount {
    pub owner: Pubkey,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub total_points: u64,
    pub last_meal_date: i64,
    pub today_meals: MealStatus,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, InitSpace)]
pub struct MealStatus {
    pub breakfast: bool,
    pub lunch: bool,
    pub dinner: bool,
    #[max_len(0)]
    pub timestamp_breakfast: Option<i64>,
    #[max_len(0)]
    pub timestamp_lunch: Option<i64>,
    #[max_len(0)]
    pub timestamp_dinner: Option<i64>,
}

impl MealStatus {
    pub fn is_complete(&self) -> bool {
        self.breakfast && self.lunch && self.dinner
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}

// Error codes
#[error_code]
pub enum FoodTrackerError {
    #[msg("This meal has already been logged today")]
    MealAlreadyLogged,

    #[msg("Current time is outside the valid window for this meal")]
    InvalidMealTime,

    #[msg("You are not authorized to access this tracker")]
    Unauthorized,
}
