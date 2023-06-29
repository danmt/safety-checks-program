use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
        CreateMetadataAccountsV3, Metadata,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::state::DataV2;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("4WJv7r8mzjydzhYRdG3yCGEmZmQT1KQUyxFrT1keaBWC");

#[program]
pub mod safety_check_manager {
    use super::*;

    pub fn create_site(ctx: Context<CreateSite>, _site_id: String) -> Result<()> {
        (*ctx.accounts.site).authority = ctx.accounts.authority.key();
        (*ctx.accounts.site).bump = *ctx.bumps.get("site").unwrap();

        Ok(())
    }

    pub fn create_inspector(ctx: Context<CreateInspector>, _site_id: String) -> Result<()> {
        (*ctx.accounts.inspector).bump = *ctx.bumps.get("inspector").unwrap();

        Ok(())
    }

    pub fn create_device(
        ctx: Context<CreateDevice>,
        _site_id: String,
        _device_id: String,
    ) -> Result<()> {
        (*ctx.accounts.device).expires_at = None;
        (*ctx.accounts.device).last_safety_check = None;
        (*ctx.accounts.device).bump = *ctx.bumps.get("device").unwrap();

        Ok(())
    }

    pub fn create_safety_check(
        ctx: Context<CreateSafetyCheck>,
        site_id: String,
        device_id: String,
        _safety_check_id: String,
        name: String,
        symbol: String,
        uri: String,
        duration_in_days: i64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let created_at = clock.unix_timestamp;
        let expires_at = created_at + duration_in_days * 86400;

        (*ctx.accounts.safety_check).inspector = ctx.accounts.inspector.key();
        (*ctx.accounts.safety_check).created_at = created_at;
        (*ctx.accounts.safety_check).duration_in_days = duration_in_days;
        (*ctx.accounts.safety_check).expires_at = expires_at;
        (*ctx.accounts.safety_check).bump = *ctx.bumps.get("safety_check").unwrap();
        (*ctx.accounts.safety_check).mint_bump = *ctx.bumps.get("safety_check_mint").unwrap();
        (*ctx.accounts.safety_check).metadata_bump =
            *ctx.bumps.get("safety_check_metadata").unwrap();
        (*ctx.accounts.safety_check).master_edition_bump =
            *ctx.bumps.get("safety_check_master_edition").unwrap();
        (*ctx.accounts.device).expires_at = Some(expires_at);
        (*ctx.accounts.device).last_safety_check = Some(ctx.accounts.safety_check.key());

        let seeds = &[
            b"device".as_ref(),
            site_id.as_bytes(),
            device_id.as_bytes(),
            &[ctx.accounts.device.bump],
        ];

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.safety_check_mint.to_account_info(),
                    to: ctx.accounts.device_safety_check_vault.to_account_info(),
                    authority: ctx.accounts.device.to_account_info(),
                },
                &[&seeds[..]],
            ),
            1,
        )?;

        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    payer: ctx.accounts.authority.to_account_info(),
                    mint: ctx.accounts.safety_check_mint.to_account_info(),
                    metadata: ctx.accounts.safety_check_metadata.to_account_info(),
                    mint_authority: ctx.accounts.device.to_account_info(),
                    update_authority: ctx.accounts.device.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &[&seeds[..]],
            ),
            DataV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            false,
            false,
            None,
        )?;

        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    edition: ctx.accounts.safety_check_master_edition.to_account_info(),
                    payer: ctx.accounts.authority.to_account_info(),
                    mint: ctx.accounts.safety_check_mint.to_account_info(),
                    metadata: ctx.accounts.safety_check_metadata.to_account_info(),
                    mint_authority: ctx.accounts.device.to_account_info(),
                    update_authority: ctx.accounts.device.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &[&seeds[..]],
            ),
            Some(1),
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(site_id: String)]
pub struct CreateSite<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = Site::SIZE,
        seeds = [b"site", site_id.as_bytes()],
        bump
    )]
    pub site: Account<'info, Site>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(site_id: String)]
pub struct CreateInspector<'info> {
    #[account(
        seeds = [b"site", site_id.as_bytes()],
        bump = site.bump
    )]
    pub site: Account<'info, Site>,
    #[account(mut, constraint = authority.key() == site.authority.key())]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = Inspector::SIZE,
        seeds = [b"inspector", site_id.as_bytes(), authority.key().as_ref()],
        bump
    )]
    pub inspector: Account<'info, Inspector>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(site_id: String, device_id: String)]
pub struct CreateDevice<'info> {
    #[account(
        seeds = [b"site", site_id.as_bytes()],
        bump = site.bump
    )]
    pub site: Account<'info, Site>,
    #[account(mut, constraint = authority.key() == site.authority.key())]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = Device::SIZE,
        seeds = [b"device", site_id.as_bytes(), device_id.as_bytes()],
        bump
    )]
    pub device: Account<'info, Device>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(
    site_id: String, 
    device_id: String, 
    safety_check_id: String, 
    name: String, 
    symbol: String, 
    uri: String, 
    duration_in_days: i64
)]
pub struct CreateSafetyCheck<'info> {
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"site", site_id.as_bytes()],
        bump = site.bump
    )]
    pub site: Account<'info, Site>,
    #[account(
        mut,
        seeds = [b"device", site_id.as_bytes(), device_id.as_bytes()],
        bump = device.bump
    )]
    pub device: Account<'info, Device>,
    #[account(
        seeds = [b"inspector", site_id.as_bytes(), authority.key().as_ref()],
        bump = inspector.bump
    )]
    pub inspector: Account<'info, Inspector>,
    #[account(
        init,
        seeds = [b"safety_check", site_id.as_bytes(), device_id.as_bytes(), safety_check_id.as_bytes()],
        bump,
        payer = authority,
        space = SafetyCheck::SIZE,
    )]
    pub safety_check: Account<'info, SafetyCheck>,
    #[account(
        init, 
        payer = authority,
        seeds = [
            b"safety_check_mint".as_ref(),
            safety_check.key().as_ref(),
        ],
        bump,
        mint::decimals = 0, 
        mint::authority = device,
        mint::freeze_authority = device,
    )]
    pub safety_check_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = safety_check_mint,
        associated_token::authority = device,
    )]
    pub device_safety_check_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            safety_check_mint.key().as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    /// CHECK: Accounts validated in the CPI to Metaplex
    pub safety_check_metadata: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            safety_check_mint.key().as_ref(),
            b"edition".as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    /// CHECK: Accounts validated in the CPI to Metaplex
    pub safety_check_master_edition: UncheckedAccount<'info>,
}

#[account]
pub struct Site {
    pub authority: Pubkey,
    pub bump: u8,
}

impl Site {
    pub const SIZE: usize = 8 + 32 + 1;
}

#[account]
pub struct Inspector {
    pub bump: u8,
}

impl Inspector {
    pub const SIZE: usize = 8 + 1;
}

#[account]
pub struct Device {
    pub expires_at: Option<i64>,
    pub last_safety_check: Option<Pubkey>,
    pub bump: u8,
}

impl Device {
    pub const SIZE: usize = 8 + 9 + 33 + 1;
}

#[account]
pub struct SafetyCheck {
    pub inspector: Pubkey,
    pub created_at: i64,
    pub duration_in_days: i64,
    pub expires_at: i64,
    pub bump: u8,
    pub mint_bump: u8,
    pub metadata_bump: u8,
    pub master_edition_bump: u8,
}

impl SafetyCheck {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 8 + 1 + 1 + 1 + 1;
}
