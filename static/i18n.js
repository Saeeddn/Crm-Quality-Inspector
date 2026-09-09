// i18n — CRM Quality Inspector Multi-Language Support
// Supported: fa (Persian, RTL), en (English, LTR)

const I18N = {
  fa: {
    // Branding
    brandTitle: 'Quality Inspector',
    brandSub: 'سامانه ارزیابی کیفیت',
    pageTitle: 'سامانه مدیریت کیفیت CRM',

    // Login
    loginTitle: 'کیفیت Inspect',
    loginSubtitle: 'سامانه ارزیابی کیفیت تعاملات CRM',
    loginUsername: 'نام کاربری',
    loginPassword: 'رمز عبور',
    loginButton: 'ورود',
    loginError: 'نام کاربری یا رمز عبور اشتباه است',

    // Navigation
    navDashboard: 'داشبورد',
    navInteractions: 'تعاملات',
    navAgents: 'کارشناسان',
    navCustomers: 'مشتریان',
    navRisk: 'سلامت مشتریان',
    navRecommendations: 'پیشنهادها',
    navIssues: 'ایرادات',
    navCoaching: 'برنامههای آموزشی',
    navCalibration: 'کالیبراسیون',
    navRubrics: 'استانداردها',
    navReport: 'گزارش کارشناس',
    navUsers: 'کاربران',
    navAudit: 'لاگ حسابرسی',

    // Common UI
    logout: 'خروج',
    loading: 'در حال بارگذاری...',
    searchPlaceholder: 'جستجو در موضوع، متن و تگها...',
    allChannels: 'همه کانالها',
    allAgents: 'همه کارشناسان',
    all: 'همه',
    allStatuses: 'همه وضعیتها',
    allSeverities: 'همه شدتها',
    allActions: 'همه عملیات',

    // Dashboard
    dashboard: 'داشبورد',
    coverageTrend: 'روند پوشش ارزیابی',
    scoreTrend: 'روند امتیازات',
    qualityDistribution: 'توزیع کیفیت',
    agentPerformance: 'عملکرد کارشناسان',
    kpiAgents: 'کارشناسان',
    kpiCustomers: 'مشتریان',
    kpiInteractions: 'تعاملات',
    kpiCoverage: 'پوشش ارزیابی',
    kpiAvgScore: 'میانگین کیفیت',
    kpiOpenIssues: 'ایرادات باز',
    kpiCriticalFail: 'شکست بحرانی',
    kpiQualityGrade: 'گرید کیفیت',
    scoredCount: function(n, total) { return n + ' از ' + total + ' تعامل ارزیابی شده'; },
    chartNoData: 'برای نمایش نمودار، حداقل ۲ ارزیابی لازم است',

    // Interactions
    interactions: 'تعاملات',
    exportCsv: 'خروجی CSV',
    newInteraction: '+ ثبت تعامل',
    colDate: 'تاریخ',
    colAgent: 'کارشناس',
    colCustomer: 'مشتری',
    colChannel: 'کانال',
    colSubject: 'موضوع',
    colQuality: 'کیفیت',
    colActions: '',
    channelPhone: 'تلفن',
    channelInperson: 'حضوری',
    channelEmail: 'ایمیل',
    channelChat: 'چت',
    channelSms: 'پیامک',
    scored: 'ارزیابیشده',
    unscored: 'ارزیابینشده',

    // Agents
    agents: 'کارشناسان',
    newAgent: '+ ثبت کارشناس',
    colName: 'نام',
    colDepartment: 'واحد',
    colPosition: 'سمت',
    colActive: 'وضعیت',
    active: 'فعال',
    inactive: 'غیرفعال',

    // Customers
    customers: 'مشتریان',
    newCustomer: '+ ثبت مشتری',
    colCustomerName: 'نام',
    colPhone: 'تلفن',
    colProduct: 'محصول',
    colSegment: 'بخش',

    // Customer Risk
    risk: 'سلامت مشتریان (Customer Risk Score)',
    riskHigh: 'ریسک بالا',
    riskMed: 'ریسک متوسط',
    riskLow: 'ریسک پایین',
    riskHighLabel: 'نیاز به تماس فوری',
    riskMedLabel: 'نیاز به پیگیری',
    riskLowLabel: 'نظارت عادی',
    totalCustomers: 'کل مشتریان',
    withRiskScore: 'با Risk Score',
    riskTableTitle: 'مشتریان پرریسک',
    colRisk: 'ریسک',
    colRiskScore: 'امتیاز',
    colLevel: 'سطح',
    colInteractions: 'تعاملات',
    colOpenIssues: 'ایرادات باز',
    colAvgScore: 'میانگین',
    colAction: 'اقدام',
    colFactors: 'دلایل',
    more: 'بیشتر',

    // Recommendations
    recommendations: 'پیشنهادها QA',

    // Issues
    issues: 'ایرادات و CAPA',
    colSeverity: 'شدت',
    colCategory: 'دسته',
    colDescription: 'شرح',
    colStatus: 'وضعیت',
    colDeadline: 'مهلت',
    severityCritical: 'بحرانی',
    severityHigh: 'بالا',
    severityMedium: 'متوسط',
    severityLow: 'پایین',
    statusOpen: 'باز',
    statusClosed: 'بسته',

    // Coaching
    coaching: 'برنامههای آموزشی (Closed-Loop QA)',
    newCoaching: '+ برنامه آموزشی',
    colCoachingStatus: 'وضعیت',
    colCoachingTheme: 'موضوع آموزشی',
    colBehaviorGap: 'شکاف رفتاری',
    colRootCause: 'علت ریشه',
    colCustomerImpact: 'اثر بر مشتری',
    colSuccessMetric: 'سنجه موفقیت',
    colFollowUpDue: 'مهلت',

    // Calibration
    calibration: 'کالیبراسیون (Blind Scoring)',
    newCalibration: '+ جلسه جدید',
    colRubric: 'روبریک',
    colScorings: 'تعداد سنجش',
    colAgreementRate: 'نرخ توافق',

    // Rubrics / KPIs
    rubrics: 'پارامترهای اندازهگیری کیفیت (KPI)',
    seedDefaultKpis: 'بارگذاری پیشفرض فارسی',
    newKpi: '+ تعریف KPI جدید',
    kpiDesc: 'امتیازدهی تعاملات بر اساس شاخصهای قابل اندازهگیری زیر به صورت خودکار محاسبه میشود.',

    // Report
    report: 'گزارش عملکرد کارشناس',

    // Users
    users: 'مدیریت کاربران',
    newUser: '+ کاربر جدید',
    userNote: 'فقط مدیر سیستم دسترسی دارد. آخرین مدیر سیستم قابل حذف نیست.',
    colUsername: 'نام کاربری',
    colRole: 'نقش',
    colCreatedAt: 'تاریخ ایجاد',
    roleAdmin: 'مدیر',
    roleUser: 'کاربر',

    // Audit
    audit: '📋 Audit Log (لاگ حسابرسی)',
    colTime: 'زمان',
    colUser: 'کاربر',
    colOperation: 'عملیات',
    colResource: 'منبع',
    colSummary: 'خلاصه',

    // Modals & Buttons
    close: 'بستن',
    save: 'ذخیره',
    cancel: 'انصراف',
    delete: 'حذف',
    confirm: 'تأیید',
    yes: 'بله',
    no: 'خیر',
    edit: 'ویرایش',
    view: 'مشاهده',

    // Toast Messages
    toastLoginExpired: 'نشست شما منقضی شده است. لطفاً دوباره وارد شوید.',
    toastLoadDashboard: 'در حال محاسبه KPI و نمودارها...',
    toastLoadRisk: 'در حال محاسبه ریسک مشتریان...',
    toastLoadAnalysis: 'در حال تحلیل ریسک و اولویتبندی...',
    toastLoading: 'در حال بارگذاری...',
    toastErrorDashboard: 'خطا در بارگذاری داشبورد:',
    toastAuthRequired: 'لطفاً ابتدا وارد شوید',
    toastUserCreated: 'کاربر با موفقیت ایجاد شد',
    toastUserDeleted: 'کاربر حذف شد',
    toastDataSaved: 'اطلاعات ذخیره شد',
    toastDataLoaded: 'اطلاعات بارگذاری شد',
    toastNoData: 'دادهای موجود نیست',

    // Legend
    legendHigh: 'بالا (تماس فوری)',
    legendMed: 'متوسط (پیگیری)',
    legendLow: 'پایین (نظارت)',

    // Empty states
    emptyTable: 'دادهای موجود نیست',

    // Channel options for filter
    channelOptions: ['تلفن', 'حضوری', 'ایمیل', 'چت', 'پیامک'],

    // Status options
    statusOptions: ['باز', 'بسته'],

    // Severity options
    severityOptions: ['بحرانی', 'بالا', 'متوسط', 'پایین'],

    // Coaching status
    coachingStatusOptions: ['پیشنویس', 'در انتظار تایید', 'تایید شده', 'در حال اجرا', 'تاییدشده با پیگیری', 'بسته شده', 'معوق/ارجاع'],

    // Calibration status
    calibrationStatusOptions: ['draft', 'scoring', 'in_session', 'completed', 'cancelled'],

    // Dashboard detail strings
    scoredCountFmt: (n, total) => n + ' از ' + total + ' تعامل ارزیابی شده',
    chartNoDataLabel: 'برای نمایش نمودار، حداقل ۲ ارزیابی لازم است',
    exportCsvBtn: 'خروجی CSV',
    newInteractionBtn: '+ ثبت تعامل',
    searchPlaceholder: 'جستجو در موضوع، متن و تگها...',
    allChannels: 'همه کانالها',
    allAgents: 'همه کارشناسان',
    allStatuses: 'همه وضعیتها',
    allSeverities: 'همه شدتها',
    channelOptions: ['تلفن', 'حضوری', 'ایمیل', 'چت', 'پیامک'],
    statusOptions: ['باز', 'بسته'],
    severityOptions: ['بحرانی', 'بالا', 'متوسط', 'پایین'],
    coachingStatusOptions: ['پیشنویس', 'در انتظار تایید', 'تایید شده', 'در حال اجرا', 'تاییدشده با پیگیری', 'بسته شده', 'معوق/ارجاع'],
    calibrationStatusOptions: ['draft', 'scoring', 'in_session', 'completed', 'cancelled'],
    reviewBtn: 'بازبینی',
    autoScoreBtn: 'ارزیابی خودکار',
    autoScoreTitle: 'ارزیابی خودکار',
    savingText: 'در حال ذخیره...',
    noKpiMsg: 'ابتدا KPI تعریف کنید یا پیشفرضها را بارگذاری کنید',
    allInteractionsScored: 'همه تعاملات ارزیابی شدهاند. عالی!',
    emptyScoreTable: 'هنوز ارزیابی نشده',
    seedDefaultKpisBtn: 'بارگذاری ۷ KPI پیشفرض',
    kpiAgentsLabel: 'کارشناسان',
    kpiCustomersLabel: 'مشتریان',
    kpiInteractionsLabel: 'تعاملات',
    kpiCoverageLabel: 'پوشش ارزیابی',
    kpiAvgScoreLabel: 'میانگین کیفیت',
    kpiOpenIssuesLabel: 'ایرادات باز',
    kpiQualityGradeLabel: 'گرید کیفیت',
    kpiAvgScoreText: 'میانگین امتیاز',
    colDate: 'تاریخ',
    colAgent: 'کارشناس',
    colCustomer: 'مشتری',
    colChannel: 'کانال',
    colSubject: 'موضوع',
    colScore: 'امتیاز',
    colActions: 'عملیات',
    exportCsvHeaders: ['شناسه','تاریخ','کارشناس','مشتری','کانال','موضوع','امتیاز','سطح','بحرانی','یادداشت'],
    viewInterBtn: 'مشاهده',
    emptyAgentMsg: 'کارشناسی یافت نشد',
    newAgentTitle: 'ثبت کارشناس',
    agentPosPlaceholder: 'مثال: کارشناس ارشد',
    agentSavedToast: 'کارشناس ثبت شد',
    editCustomerTitle: function(name) { return 'ویرایش مشتری: ' + name; },
    customerUpdatedToast: 'مشتری بهروزرسانی شد',
    newCustomerTitle: 'ثبت مشتری',
    customerSavedToast: 'مشتری ثبت شد',
    selectAgentPlaceholder: '— کارشناسی نیست —',
    reportSelectPlaceholder: 'انتخاب کارشناس...',
    dateCol: 'تاریخ',
    scoreCol: 'امتیاز',
    levelCol: 'سطح',
    statusCol: 'وضعیت',
    coachingThemePlaceholder: 'مثلاً احوالپرسی آغازین',
    behaviorGapPlaceholder: 'مثلاً عدم احوالپرسی با مشتری',
    customerImpactPlaceholder: 'مثلاً کاهش رضایت',
    requiredAgentError: 'کارشناس الزامی است',
    requiredSubjectTranscriptError: 'موضوع و متن الزامی است',
    manualKpiOption: 'دستی (امتیاز توسط ارزیاب)',
    autoScoreResultSaved: function(score, level) { return `امتیاز ${score.toFixed(1)} (${level}) ذخیره شد`; },
    kpiAgentsLabel: 'کارشناسان',
    kpiCustomersLabel: 'مشتریان',
    kpiInteractionsLabel: 'تعاملات',
    kpiCoverageLabel: 'پوشش ارزیابی',
    kpiAvgScoreLabel: 'میانگین کیفیت',
    kpiOpenIssuesLabel: 'ایرادات باز',
    kpiQualityGradeLabel: 'گرید کیفیت',
    kpiAvgScoreText: 'میانگین امتیاز',
    colDate: 'تاریخ',
    colAgent: 'کارشناس',
    colCustomer: 'مشتری',
    colChannel: 'کانال',
    colSubject: 'موضوع',
    colScore: 'امتیاز',
    colActions: 'عملیات',
    exportCsvHeaders: ['شناسه','تاریخ','کارشناس','مشتری','کانال','موضوع','امتیاز','سطح','بحرانی','یادداشت'],
    viewInterBtn: 'مشاهده',
    emptyAgentMsg: 'کارشناسی یافت نشد',
    newAgentTitle: 'ثبت کارشناس',
    agentPosPlaceholder: 'مثال: کارشناس ارشد',
    agentSavedToast: 'کارشناس ثبت شد',
    editCustomerTitle: (name) => 'ویرایش مشتری: ' + name,
    customerUpdatedToast: 'مشتری بهروزرسانی شد',
    newCustomerTitle: 'ثبت مشتری',
    customerSavedToast: 'مشتری ثبت شد',
    selectAgentPlaceholder: '— کارشناسی نیست —',
    reportSelectPlaceholder: 'انتخاب کارشناس...',
    dateCol: 'تاریخ',
    scoreCol: 'امتیاز',
    levelCol: 'سطح',
    statusCol: 'وضعیت',
    coachingThemePlaceholder: 'مثلاً احوالپرسی آغازین',
    behaviorGapPlaceholder: 'مثلاً عدم احوالپرسی با مشتری',
    customerImpactPlaceholder: 'مثلاً کاهش رضایت',
    requiredAgentError: 'کارشناس الزامی است',
    requiredSubjectTranscriptError: 'موضوع و متن الزامی است',
    manualKpiOption: 'دستی (امتیاز توسط ارزیاب)',
    autoScoreResultSaved: (score, level) => `امتیاز ${score.toFixed(1)} (${level}) ذخیره شد`,
    transcriptPlaceholder: 'مثال: سلام. مشتری با عصبانیت شکایت کرد که ...',
    colSessionName: 'نام جلسه',
    colStandard: 'استاندارد',
    colSamples: 'نمونهها',
    colAgreement: 'توافق',
    colDeadline: 'مهلت',
    colName: 'نام',
    colDepartment: 'واحد',
    colPosition: 'سمت',
    kpisLoadedToast: (n) => n + ' KPI بارگذاری شد',
    coachLoadedToast: 'خطa در بارگذاری برنامههای آموزشی:',
    calLoadedToast: 'خطa در بارگذاری کالیبراسیون:',
    coachLoadLabel: 'در حال بارگذاری برنامههای آموزشی...',
    calLoadLabel: 'در حال بارگذاری کالیبراسیون...',
    calSessionLoadLabel: 'در حال بارگذاری جلسه...',
    createCalLabel: 'در حال ایجاد جلسه...',
    submitScoresLabel: 'در حال ثبت امتیازها...',
    createCoachLabel: 'در حال ایجاد برنامه...',
    loadingAllLabel: 'در حال بارگذاری...',
    enteringLoginLabel: 'در حال ورود...',
    kpiAgentsLabel: 'Agents',
    kpiCustomersLabel: 'Customers',
    kpiInteractionsLabel: 'Interactions',
    kpiCoverageLabel: 'Assessment Coverage',
    kpiAvgScoreLabel: 'Avg Quality Score',
    kpiOpenIssuesLabel: 'Open Issues',
    kpiQualityGradeLabel: 'Quality Grade',
    kpiAvgScoreText: 'Average Score',
    colDate: 'Date',
    colAgent: 'Agent',
    colCustomer: 'Customer',
    colChannel: 'Channel',
    colSubject: 'Subject',
    colScore: 'Score',
    colActions: 'Actions',
    exportCsvHeaders: ['ID','Date','Agent','Customer','Channel','Subject','Score','Level','Critical','Notes'],
    viewInterBtn: 'View',
    emptyAgentMsg: 'No agents found',
    newAgentTitle: 'Register Agent',
    agentPosPlaceholder: 'e.g. Senior Agent',
    agentSavedToast: 'Agent registered',
    editCustomerTitle: function(name) { return 'Edit Customer: ' + name; },
    customerUpdatedToast: 'Customer updated',
    newCustomerTitle: 'Add Customer',
    customerSavedToast: 'Customer saved',
    selectAgentPlaceholder: '— No agent —',
    reportSelectPlaceholder: 'Select agent...',
    dateCol: 'Date',
    scoreCol: 'Score',
    levelCol: 'Level',
    statusCol: 'Status',
    coachingThemePlaceholder: 'e.g. Initial greeting',
    behaviorGapPlaceholder: 'e.g. No greeting to customer',
    customerImpactPlaceholder: 'e.g. Reduced satisfaction',
    requiredAgentError: 'Agent is required',
    requiredSubjectTranscriptError: 'Subject and transcript are required',
    manualKpiOption: 'Manual (scored by evaluator)',
    autoScoreResultSaved: function(score, level) { return `Score ${score.toFixed(1)} (${level}) saved`; },
    kpiAgentsLabel: 'Agents',
    kpiCustomersLabel: 'Customers',
    kpiInteractionsLabel: 'Interactions',
    kpiCoverageLabel: 'Assessment Coverage',
    kpiAvgScoreLabel: 'Avg Quality Score',
    kpiOpenIssuesLabel: 'Open Issues',
    kpiQualityGradeLabel: 'Quality Grade',
    kpiAvgScoreText: 'Average Score',
    colDate: 'Date',
    colAgent: 'Agent',
    colCustomer: 'Customer',
    colChannel: 'Channel',
    colSubject: 'Subject',
    colScore: 'Score',
    colActions: 'Actions',
    exportCsvHeaders: ['ID','Date','Agent','Customer','Channel','Subject','Score','Level','Critical','Notes'],
    viewInterBtn: 'View',
    emptyAgentMsg: 'No agents found',
    newAgentTitle: 'Register Agent',
    agentPosPlaceholder: 'e.g. Senior Agent',
    agentSavedToast: 'Agent registered',
    editCustomerTitle: (name) => 'Edit Customer: ' + name,
    customerUpdatedToast: 'Customer updated',
    newCustomerTitle: 'Add Customer',
    customerSavedToast: 'Customer saved',
    selectAgentPlaceholder: '-- No agent --',
    reportSelectPlaceholder: 'Select agent...',
    dateCol: 'Date',
    scoreCol: 'Score',
    levelCol: 'Level',
    statusCol: 'Status',
    coachingThemePlaceholder: 'e.g. Initial greeting',
    behaviorGapPlaceholder: 'e.g. No greeting to customer',
    customerImpactPlaceholder: 'e.g. Reduced satisfaction',
    requiredAgentError: 'Agent is required',
    requiredSubjectTranscriptError: 'Subject and transcript are required',
    manualKpiOption: 'Manual (scored by evaluator)',
    autoScoreResultSaved: (score, level) => `Score ${score.toFixed(1)} (${level}) saved`,
    transcriptPlaceholder: 'e.g. Hello. The customer complained angrily about...',
    colSessionName: 'Session Name',
    colStandard: 'Rubric',
    colSamples: 'Samples',
    colAgreement: 'Agreement',
    colDeadline: 'Due',
    colName: 'Name',
    colDepartment: 'Department',
    colPosition: 'Position',
    calStatusMap: {'draft': 'پیشنویس', 'scoring': 'در حال امتیازدهی', 'in_session': 'در جلسه', 'completed': 'تکمیلشده', 'cancelled': 'لغو شده'},
    coachStatusMap: {'draft': 'پیشنویس', 'pending_acknowledgement': 'در انتظار تایید', 'acknowledged': 'تایید شده', 'in_progress': 'در حال اجرا', 'verified': 'تاییدشده با پیگیری', 'closed': 'بسته شده', 'escalated': 'معوق/ارجاع'},

    // Detail strings
    criticalFailLabel: 'شکست بحرانی',
    evaluatorLabel: 'ارزیاب',
    noKpiDefined: 'هیچ KPI تعریف نشده است',
    criticalFailNote: 'شکست بحرانی (اگر نمره کمتر از ۶۰ باشد، کل interaction شکست میخورد)',    isOpenStatus: 'باز',
    isClosedStatus: 'بسته',
    isActiveAgent: 'فعال',
    isInactiveAgent: 'غیرفعال',
    csvDownloadedToast: 'فایل CSV دانلود شد',
    yesText: 'بله',
    noText: 'خیر',
        // Priority pill labels
    priorityHigh: 'بالا',
    priorityMedium: 'متوسط',
    priorityLow: 'پایین',
    priorityLabel: 'اولویت',
    
    // Severity pill labels
    severityCritical: 'بحرانی',
    severityHigh: 'بالا',
    severityMedium: 'متوسط',
    severityLow: 'پایین',
    
    // Customer risk labels
    riskHigh: 'ریسک بالا',
    riskHighSub: 'نیاز به تماس فوری',
    riskMed: 'ریسک متوسط',
    riskMedSub: 'نیاز به پیگیری',
    riskLow: 'ریسک پایین',
    riskLowSub: 'نظارت عادی',
    totalCustomersLabel: 'کل مشتریان',
    withRiskScoreLabel: 'با Risk Score',
    noDataAvailable: 'دادهای موجود نیست',
    moreLabel: 'بیشتر',
    
    // Chart labels
    avgQualityScore: 'میانگین امتیاز کیفیت',
    healthyLabel: 'سالم (≥80)',
    needImproveLabel: 'نیازمند بهبود (60-80)',
    criticalLabel: 'بحرانی (<60)',
    noRecordLabel: 'بدون رکورد',
    customPageSize: 'سفارشی...',
    pageSizePlaceholder: 'تعداد',
    paginationInfo: function(start, end, total, page, totalPages) { 
        return `نمایش <b>${start}–${end}</b> از <b>${total}</b> رکورد (صفحه ${page} از ${totalPages})`; 
    },
    firstPage: '« اول',
    prevPage: '‹ قبلی',
    nextPage: 'بعدی ›',
    lastPage: 'آخر »',
    pageSizeLabel: 'تعداد در صفحه:',
    pageSizePrompt: 'تعداد در صفحه را وارد کنید (۱ تا ۱۰۰۰):',
    allAgentsLabel: 'همه کارشناسان',
    autoScoreProgress: function(count) { return `در حال اندازهگیری خودکار ${count} KPI...`; },
    interactionDetailTitle: 'جزئیات تعامل',
    transcriptLabel: 'متن مکالمه:',
    criticalFailLabel: 'شکست بحرانی',
    evaluatorNote: 'یادداشت ارزیاب (اختیاری)',
    notesPlaceholder: 'نکات تکمیلی شما',
    saveToDashboard: 'ذخیره در داشبورد',
    cancelBtn: 'انصراف',
    saveBtn: 'ذخیره',
    noActiveKpiError: 'هیچ KPI فعالی',
    allScoredMsg: 'همه تعاملات ارزیابی شدهاند. عالی!',
    expertCustomerChannel: function(agent, customer, channel) { 
        return `کارشناس: ${agent || '-'} | مشتری: ${customer || '-'} | کانال: ${channel}`; 
    },
    modalCloseBtn: 'بستن',
    editUserLabel: 'تغییر رمز / نقش',
    deleteBtn: 'حذف',
    cannotDeleteSelf: 'نمیتوانید خودتان را حذف کنید',
    confirmDelete: function(username) { return `کاربر "${username}" حذف شود؟`; },
    userDeletedToast: 'کاربر حذف شد',
    newUserTitle: 'ایجاد کاربر جدید',
    usernameLabel: 'نام کاربری (حداقل ۳ کاراکتر)',
    passwordLabel: 'رمز عبور (حداقل ۴ کاراکتر)',
    adminAccessLabel: 'دسترسی مدیر سیستم',
    createBtn: 'ایجاد',
    usernameRequired: 'نام کاربری الزامی است',
    userCreatedToast: 'کاربر ایجاد شد',
    editUserTitle: function(username) { return `ویرایش کاربر: ${username}`; },
    newPasswordLabel: 'رمز عبور جدید (خالی = بدون تغییر)',
    updatedToast: 'کاربر بهroزرسانی شد',
    onlyAdminAccessible: 'فقط مدیر سیستم دسترسی دارد',
        // Product types
    productBank: 'بانک',
    productInsurance: 'بیمه',
    productInvestment: 'سرمایهگذاری',
    productLoan: 'وام',
    
    // Buttons
    editBtn: 'ویرایش',
    deleteBtn: 'حذف',
    saveBtn: 'ذخیره',
    cancelBtn: 'انصراف',
    closeBtn: 'بستن',
    reportBtn: 'گزارش',
    
    // Messages
    emptyCustomerMsg: 'مشتری یافت نشد',
    customerDeletedToast: 'حذف شد',
    confirmDelete: function(name) { return `حذف ${name}؟`; },
    isActiveAgent: 'فعال',
    isInactiveAgent: 'غیرفعال',
    
    // Risk levels
    riskHigh: 'ریسک بالا',
    riskMed: 'ریسک متوسط',
    riskLow: 'ریسک پایین',
    riskHighSub: 'نیاز به تماس فوری',
    riskMedSub: 'نیاز به پیگیری',
    riskLowSub: 'نظارت عادی',
    totalCustomersLabel: 'کل مشتریان',
    withRiskScoreLabel: 'با Risk Score',
    noDataAvailable: 'دادهای موجود نیست',
    moreLabel: 'بیشتر',
    // More UI strings
    pageInfoLabel: function(s,e,t,p,tp) { return `نمایش <b>${s}–${e}</b> از <b>${t}</b> رکورد (صفحه ${p} از ${tp})`; },
    pageSizeLabel: 'تعداد در صفحه:',
    pageSizePrompt: 'تعداد در صفحه را وارد کنید (۱ تا ۱۰۰۰):',
    allAgentsOpt: 'همه کارشناسان',
    autoScoreProgress: function(n) { return `در حال اندازهگیری خودکار ${n} KPI...`; },
    interactionDetailTitle: 'جزئیات تعامل',
    transcriptLabel: 'متن مکالمه:',
    criticalFailLabel2: 'شکست بحرانی',
    evaluatorNote: 'یادداشت ارزیاب (اختیاری)',
    notesPlaceholder: 'نکات تکمیلی شما',
    saveToDashboardBtn: 'ذخیره در داشبورد',
    cancelBtn2: 'انصراف',
    noActiveKpi: 'هیچ KPI فعالی',
    kpiMsg: 'ابتدا KPI تعریف کنید یا پیشفرضها را بارگذاری کنید',
    expertCustomerChannel: function(a,c,ch) { return `کارشناس: ${a || '-'} | مشتری: ${c || '-'} | کانال: ${ch}`; },
    riskLabel: 'ریسک',
    riskFactorsLabel: 'دلایل ریسک:',
    suggestedActionLabel: 'اقدام پیشنهادی:',
    autoScoreBtn2: 'ارزیابی خودکار',
    activeLabel: 'فعال',
    inactiveLabel: 'غیرفعال',
    reportBtn2: 'گزارش',
    toggleToast: function(active) { return active ? 'فعال شد' : 'غیرفعال شد'; },
    agentNameLabel: 'نام و نام خانوادگی',
    agentDeptLabel: 'واحد',
    agentPosLabel: 'سمت',
    agentSavedToast: 'کارشناس ثبت شد',
    editCustLabel: 'ویرایش',
    deleteCustLabel: 'حذف',
    segmentLabel: 'سطح',
    segmentNormal: 'عادی',
    segmentImportant: 'مهم',
    notesFieldLabel: 'یادداشت',
    customerUpdatedToast: 'مشتری بهroزرسانی شد',
    newCustomerName: 'نام',
    newCustomerPhone: 'تلفن',
    newCustomerProduct: 'محصول',
    newCustomerSeg: 'بخش',
    normalOpt: 'عادی',
    vipOpt: 'VIP',
    corpOpt: 'شرکتی',
    customerCreatedToast: 'مشتری ثبت شد',
    // Score pills
    scoreNotEvaluated: 'ارزیابینشده',
    emptyTable: 'دادهای موجود نیست',


    // Buttons
    editBtn: 'Edit',
    deleteBtn: 'Delete',
    saveBtn: 'Save',
    cancelBtn: 'Cancel',
    closeBtn: 'Close',
    reportBtn: 'Report',
    
    // Messages
    emptyCustomerMsg: 'No customers found',
    customerDeletedToast: 'Deleted',
    confirmDelete: (name) => `Delete ${name}?`,
    isActiveAgent: 'Active',
    isInactiveAgent: 'Inactive',
    
    // Product types
    productBank: 'Bank',
    productInsurance: 'Insurance',
    productInvestment: 'Investment',
    productLoan: 'Loan',
    // Error messages
    invalidResponse: 'پاسخ نامعتبر',

    // RTL/LTR
    dir: 'rtl',
    lang: 'fa',
  },

  en: {
    // Branding
    brandTitle: 'Quality Inspector',
    brandSub: 'Quality Assessment System',
    pageTitle: 'CRM Quality Management System',

    // Login
    loginTitle: 'Quality Inspector',
    loginSubtitle: 'CRM Interaction Quality Assessment',
    loginUsername: 'Username',
    loginPassword: 'Password',
    loginButton: 'Login',
    loginError: 'Invalid username or password',

    // Navigation
    navDashboard: 'Dashboard',
    navInteractions: 'Interactions',
    navAgents: 'Agents',
    navCustomers: 'Customers',
    navRisk: 'Customer Health',
    navRecommendations: 'Recommendations',
    navIssues: 'Issues',
    navCoaching: 'Coaching Plans',
    navCalibration: 'Calibration',
    navRubrics: 'Standards',
    navReport: 'Agent Report',
    navUsers: 'Users',
    navAudit: 'Audit Log',

    // Common UI
    logout: 'Logout',
    loading: 'Loading...',
    searchPlaceholder: 'Search subject, transcript, tags...',
    allChannels: 'All Channels',
    allAgents: 'All Agents',
    all: 'All',
    allStatuses: 'All Statuses',
    allSeverities: 'All Severities',
    allActions: 'All Actions',

    // Dashboard
    dashboard: 'Dashboard',
    coverageTrend: 'Assessment Coverage Trend',
    scoreTrend: 'Score Trend',
    qualityDistribution: 'Quality Distribution',
    agentPerformance: 'Agent Performance',
    kpiAgents: 'Agents',
    kpiCustomers: 'Customers',
    kpiInteractions: 'Interactions',
    kpiCoverage: 'Assessment Coverage',
    kpiAvgScore: 'Avg Quality Score',
    kpiOpenIssues: 'Open Issues',
    kpiCriticalFail: 'Critical Failures',
    kpiQualityGrade: 'Quality Grade',
    scoredCount: function(n, total) { return n + ' of ' + total + ' interactions scored'; },
    chartNoData: 'At least 2 scores required to display chart',

    // Interactions
    interactions: 'Interactions',
    exportCsv: 'Export CSV',
    newInteraction: '+ New Interaction',
    colDate: 'Date',
    colAgent: 'Agent',
    colCustomer: 'Customer',
    colChannel: 'Channel',
    colSubject: 'Subject',
    colQuality: 'Quality',
    colActions: '',
    channelPhone: 'Phone',
    channelInperson: 'In-Person',
    channelEmail: 'Email',
    channelChat: 'Chat',
    channelSms: 'SMS',
    scored: 'Scored',
    unscored: 'Unscored',

    // Agents
    agents: 'Agents',
    newAgent: '+ New Agent',
    colName: 'Name',
    colDepartment: 'Department',
    colPosition: 'Position',
    colActive: 'Status',
    active: 'Active',
    inactive: 'Inactive',

    // Customers
    customers: 'Customers',
    newCustomer: '+ New Customer',
    colCustomerName: 'Name',
    colPhone: 'Phone',
    colProduct: 'Product',
    colSegment: 'Segment',

    // Customer Risk
    risk: 'Customer Health (Customer Risk Score)',
    riskHigh: 'High Risk',
    riskMed: 'Medium Risk',
    riskLow: 'Low Risk',
    riskHighLabel: 'Immediate contact needed',
    riskMedLabel: 'Follow-up required',
    riskLowLabel: 'Normal monitoring',
    totalCustomers: 'Total Customers',
    withRiskScore: 'With Risk Score',
    riskTableTitle: 'High-Risk Customers',
    colRisk: 'Risk',
    colRiskScore: 'Score',
    colLevel: 'Level',
    colInteractions: 'Interactions',
    colOpenIssues: 'Open Issues',
    colAvgScore: 'Average',
    colAction: 'Action',
    colFactors: 'Reasons',
    more: 'more',

    // Recommendations
    recommendations: 'QA Recommendations',

    // Issues
    issues: 'Issues & CAPA',
    colSeverity: 'Severity',
    colCategory: 'Category',
    colDescription: 'Description',
    colStatus: 'Status',
    colDeadline: 'Due Date',
    severityCritical: 'Critical',
    severityHigh: 'High',
    severityMedium: 'Medium',
    severityLow: 'Low',
    statusOpen: 'Open',
    statusClosed: 'Closed',

    // Coaching
    coaching: 'Coaching Plans (Closed-Loop QA)',
    newCoaching: '+ New Coaching Plan',
    colCoachingStatus: 'Status',
    colCoachingTheme: 'Coaching Theme',
    colBehaviorGap: 'Behavior Gap',
    colRootCause: 'Root Cause',
    colCustomerImpact: 'Customer Impact',
    colSuccessMetric: 'Success Metric',
    colFollowUpDue: 'Due Date',

    // Calibration
    calibration: 'Calibration (Blind Scoring)',
    newCalibration: '+ New Session',
    colRubric: 'Rubric',
    colScorings: 'Scorings',
    colAgreementRate: 'Agreement Rate',

    // Rubrics / KPIs
    rubrics: 'Quality Measurement Parameters (KPI)',
    seedDefaultKpis: 'Load Default KPIs',
    newKpi: '+ New KPI',
    kpiDesc: 'Interaction scoring is automatically calculated based on the following measurable indicators.',

    // Report
    report: 'Agent Performance Report',

    // Users
    users: 'User Management',
    newUser: '+ New User',
    userNote: 'Only system administrators have access. The last admin cannot be deleted.',
    colUsername: 'Username',
    colRole: 'Role',
    colCreatedAt: 'Created At',
    roleAdmin: 'Admin',
    roleUser: 'User',

    // Audit
    audit: '📋 Audit Log',
    colTime: 'Time',
    colUser: 'User',
    colOperation: 'Operation',
    colResource: 'Resource',
    colSummary: 'Summary',

    // Modals & Buttons
    close: 'Close',
    save: 'Save',
    cancel: 'Cancel',
    delete: 'Delete',
    confirm: 'Confirm',
    yes: 'Yes',
    no: 'No',
    edit: 'Edit',
    view: 'View',

    // Toast Messages
    toastLoginExpired: 'Your session has expired. Please log in again.',
    toastLoadDashboard: 'Calculating KPIs and charts...',
    toastLoadRisk: 'Calculating customer risks...',
    toastLoadAnalysis: 'Analyzing risks and prioritizing...',
    toastLoading: 'Loading...',
    toastErrorDashboard: 'Error loading dashboard:',
    toastAuthRequired: 'Please log in first',
    toastUserCreated: 'User created successfully',
    toastUserDeleted: 'User deleted',
    toastDataSaved: 'Data saved',
    toastDataLoaded: 'Data loaded',
    toastNoData: 'No data available',

    // Legend
    legendHigh: 'High (Urgent)',
    legendMed: 'Medium (Follow-up)',
    legendLow: 'Low (Monitor)',

    // Empty states
    emptyTable: 'No data available',

    // Channel options for filter
    channelOptions: ['Phone', 'In-Person', 'Email', 'Chat', 'SMS'],

    // Status options
    statusOptions: ['Open', 'Closed'],

    // Severity options
    severityOptions: ['Critical', 'High', 'Medium', 'Low'],

    // Coaching status
    coachingStatusOptions: ['Draft', 'Pending Acknowledgement', 'Acknowledged', 'In Progress', 'Verified with Follow-up', 'Closed', 'Escalated'],

    // Calibration status
    calibrationStatusOptions: ['draft', 'scoring', 'in_session', 'completed', 'cancelled'],

    // Dashboard detail strings
    scoredCountFmt: (n, total) => n + ' of ' + total + ' interactions scored',
    chartNoDataLabel: 'At least 2 scores required to display chart',
    exportCsvBtn: 'Export CSV',
    newInteractionBtn: '+ New Interaction',
    searchPlaceholder: 'Search subject, transcript, tags...',
    allChannels: 'All Channels',
    allAgents: 'All Agents',
    allStatuses: 'All Statuses',
    allSeverities: 'All Severities',
    channelOptions: ['Phone', 'In-Person', 'Email', 'Chat', 'SMS'],
    statusOptions: ['Open', 'Closed'],
    severityOptions: ['Critical', 'High', 'Medium', 'Low'],
    coachingStatusOptions: ['Draft', 'Pending Acknowledgement', 'Acknowledged', 'In Progress', 'Verified with Follow-up', 'Closed', 'Escalated'],
    calibrationStatusOptions: ['draft', 'scoring', 'in_session', 'completed', 'cancelled'],
    reviewBtn: 'Review',
    autoScoreBtn: 'Auto Score',
    autoScoreTitle: 'Auto Scoring',
    savingText: 'Saving...',
    noKpiMsg: 'Define KPIs first or load defaults',
    allInteractionsScored: 'All interactions scored. Great job!',
    emptyScoreTable: 'No scores yet',
    seedDefaultKpisBtn: 'Load Default KPIs',
    kpiAgentsLabel: 'کارشناسان',
    kpiCustomersLabel: 'مشتریان',
    kpiInteractionsLabel: 'تعاملات',
    kpiCoverageLabel: 'پوشش ارزیابی',
    kpiAvgScoreLabel: 'میانگین کیفیت',
    kpiOpenIssuesLabel: 'ایرادات باز',
    kpiQualityGradeLabel: 'گرید کیفیت',
    kpiAvgScoreText: 'میانگین امتیاز',
    colDate: 'تاریخ',
    colAgent: 'کارشناس',
    colCustomer: 'مشتری',
    colChannel: 'کانال',
    colSubject: 'موضوع',
    colScore: 'امتیاز',
    colActions: 'عملیات',
    exportCsvHeaders: ['شناسه','تاریخ','کارشناس','مشتری','کانال','موضوع','امتیاز','سطح','بحرانی','یادداشت'],
    viewInterBtn: 'مشاهده',
    emptyAgentMsg: 'کارشناسی یافت نشد',
    newAgentTitle: 'ثبت کارشناس',
    agentPosPlaceholder: 'مثال: کارشناس ارشد',
    agentSavedToast: 'کارشناس ثبت شد',
    editCustomerTitle: function(name) { return 'ویرایش مشتری: ' + name; },
    customerUpdatedToast: 'مشتری بهروزرسانی شد',
    newCustomerTitle: 'ثبت مشتری',
    customerSavedToast: 'مشتری ثبت شد',
    selectAgentPlaceholder: '— کارشناسی نیست —',
    reportSelectPlaceholder: 'انتخاب کارشناس...',
    dateCol: 'تاریخ',
    scoreCol: 'امتیاز',
    levelCol: 'سطح',
    statusCol: 'وضعیت',
    coachingThemePlaceholder: 'مثلاً احوالپرسی آغازین',
    behaviorGapPlaceholder: 'مثلاً عدم احوالپرسی با مشتری',
    customerImpactPlaceholder: 'مثلاً کاهش رضایت',
    requiredAgentError: 'کارشناس الزامی است',
    requiredSubjectTranscriptError: 'موضوع و متن الزامی است',
    manualKpiOption: 'دستی (امتیاز توسط ارزیاب)',
    autoScoreResultSaved: function(score, level) { return `امتیاز ${score.toFixed(1)} (${level}) ذخیره شد`; },
    kpiAgentsLabel: 'کارشناسان',
    kpiCustomersLabel: 'مشتریان',
    kpiInteractionsLabel: 'تعاملات',
    kpiCoverageLabel: 'پوشش ارزیابی',
    kpiAvgScoreLabel: 'میانگین کیفیت',
    kpiOpenIssuesLabel: 'ایرادات باز',
    kpiQualityGradeLabel: 'گرید کیفیت',
    kpiAvgScoreText: 'میانگین امتیاز',
    colDate: 'تاریخ',
    colAgent: 'کارشناس',
    colCustomer: 'مشتری',
    colChannel: 'کانال',
    colSubject: 'موضوع',
    colScore: 'امتیاز',
    colActions: 'عملیات',
    exportCsvHeaders: ['شناسه','تاریخ','کارشناس','مشتری','کانال','موضوع','امتیاز','سطح','بحرانی','یادداشت'],
    viewInterBtn: 'مشاهده',
    emptyAgentMsg: 'کارشناسی یافت نشد',
    newAgentTitle: 'ثبت کارشناس',
    agentPosPlaceholder: 'مثال: کارشناس ارشد',
    agentSavedToast: 'کارشناس ثبت شد',
    editCustomerTitle: (name) => 'ویرایش مشتری: ' + name,
    customerUpdatedToast: 'مشتری بهروزرسانی شد',
    newCustomerTitle: 'ثبت مشتری',
    customerSavedToast: 'مشتری ثبت شد',
    selectAgentPlaceholder: '— کارشناسی نیست —',
    reportSelectPlaceholder: 'انتخاب کارشناس...',
    dateCol: 'تاریخ',
    scoreCol: 'امتیاز',
    levelCol: 'سطح',
    statusCol: 'وضعیت',
    coachingThemePlaceholder: 'مثلاً احوالپرسی آغازین',
    behaviorGapPlaceholder: 'مثلاً عدم احوالپرسی با مشتری',
    customerImpactPlaceholder: 'مثلاً کاهش رضایت',
    requiredAgentError: 'کارشناس الزامی است',
    requiredSubjectTranscriptError: 'موضوع و متن الزامی است',
    manualKpiOption: 'دستی (امتیاز توسط ارزیاب)',
    autoScoreResultSaved: (score, level) => `امتیاز ${score.toFixed(1)} (${level}) ذخیره شد`,
    transcriptPlaceholder: 'مثال: سلام. مشتری با عصبانیت شکایت کرد که ...',
    colSessionName: 'نام جلسه',
    colStandard: 'استاندارد',
    colSamples: 'نمونهها',
    colAgreement: 'توافق',
    colDeadline: 'مهلت',
    colName: 'نام',
    colDepartment: 'واحد',
    colPosition: 'سمت',
    kpisLoadedToast: (n) => n + ' KPIs loaded',
    coachLoadedToast: 'Error loading coaching plans:',
    calLoadedToast: 'Error loading calibration:',
    coachLoadLabel: 'Loading coaching plans...',
    calLoadLabel: 'Loading calibration...',
    calSessionLoadLabel: 'Loading session...',
    createCalLabel: 'Creating session...',
    submitScoresLabel: 'Submitting scores...',
    createCoachLabel: 'Creating plan...',
    loadingAllLabel: 'Loading...',
    enteringLoginLabel: 'Logging in...',
    kpiAgentsLabel: 'Agents',
    kpiCustomersLabel: 'Customers',
    kpiInteractionsLabel: 'Interactions',
    kpiCoverageLabel: 'Assessment Coverage',
    kpiAvgScoreLabel: 'Avg Quality Score',
    kpiOpenIssuesLabel: 'Open Issues',
    kpiQualityGradeLabel: 'Quality Grade',
    kpiAvgScoreText: 'Average Score',
    colDate: 'Date',
    colAgent: 'Agent',
    colCustomer: 'Customer',
    colChannel: 'Channel',
    colSubject: 'Subject',
    colScore: 'Score',
    colActions: 'Actions',
    exportCsvHeaders: ['ID','Date','Agent','Customer','Channel','Subject','Score','Level','Critical','Notes'],
    viewInterBtn: 'View',
    emptyAgentMsg: 'No agents found',
    newAgentTitle: 'Register Agent',
    agentPosPlaceholder: 'e.g. Senior Agent',
    agentSavedToast: 'Agent registered',
    editCustomerTitle: function(name) { return 'Edit Customer: ' + name; },
    customerUpdatedToast: 'Customer updated',
    newCustomerTitle: 'Add Customer',
    customerSavedToast: 'Customer saved',
    selectAgentPlaceholder: '— No agent —',
    reportSelectPlaceholder: 'Select agent...',
    dateCol: 'Date',
    scoreCol: 'Score',
    levelCol: 'Level',
    statusCol: 'Status',
    coachingThemePlaceholder: 'e.g. Initial greeting',
    behaviorGapPlaceholder: 'e.g. No greeting to customer',
    customerImpactPlaceholder: 'e.g. Reduced satisfaction',
    requiredAgentError: 'Agent is required',
    requiredSubjectTranscriptError: 'Subject and transcript are required',
    manualKpiOption: 'Manual (scored by evaluator)',
    autoScoreResultSaved: function(score, level) { return `Score ${score.toFixed(1)} (${level}) saved`; },
    kpiAgentsLabel: 'Agents',
    kpiCustomersLabel: 'Customers',
    kpiInteractionsLabel: 'Interactions',
    kpiCoverageLabel: 'Assessment Coverage',
    kpiAvgScoreLabel: 'Avg Quality Score',
    kpiOpenIssuesLabel: 'Open Issues',
    kpiQualityGradeLabel: 'Quality Grade',
    kpiAvgScoreText: 'Average Score',
    colDate: 'Date',
    colAgent: 'Agent',
    colCustomer: 'Customer',
    colChannel: 'Channel',
    colSubject: 'Subject',
    colScore: 'Score',
    colActions: 'Actions',
    exportCsvHeaders: ['ID','Date','Agent','Customer','Channel','Subject','Score','Level','Critical','Notes'],
    viewInterBtn: 'View',
    emptyAgentMsg: 'No agents found',
    newAgentTitle: 'Register Agent',
    agentPosPlaceholder: 'e.g. Senior Agent',
    agentSavedToast: 'Agent registered',
    editCustomerTitle: (name) => 'Edit Customer: ' + name,
    customerUpdatedToast: 'Customer updated',
    newCustomerTitle: 'Add Customer',
    customerSavedToast: 'Customer saved',
    selectAgentPlaceholder: '-- No agent --',
    reportSelectPlaceholder: 'Select agent...',
    dateCol: 'Date',
    scoreCol: 'Score',
    levelCol: 'Level',
    statusCol: 'Status',
    coachingThemePlaceholder: 'e.g. Initial greeting',
    behaviorGapPlaceholder: 'e.g. No greeting to customer',
    customerImpactPlaceholder: 'e.g. Reduced satisfaction',
    requiredAgentError: 'Agent is required',
    requiredSubjectTranscriptError: 'Subject and transcript are required',
    manualKpiOption: 'Manual (scored by evaluator)',
    autoScoreResultSaved: (score, level) => `Score ${score.toFixed(1)} (${level}) saved`,
    transcriptPlaceholder: 'e.g. Hello. The customer complained angrily about...',
    colSessionName: 'Session Name',
    colStandard: 'Rubric',
    colSamples: 'Samples',
    colAgreement: 'Agreement',
    colDeadline: 'Due',
    colName: 'Name',
    colDepartment: 'Department',
    colPosition: 'Position',
    calStatusMap: {'draft': 'Draft', 'scoring': 'Scoring', 'in_session': 'In Session', 'completed': 'Completed', 'cancelled': 'Cancelled'},
    coachStatusMap: {'draft': 'Draft', 'pending_acknowledgement': 'Pending Acknowledgement', 'acknowledged': 'Acknowledged', 'in_progress': 'In Progress', 'verified': 'Verified with Follow-up', 'closed': 'Closed', 'escalated': 'Escalated'},

    // Detail strings
    criticalFailLabel: 'Critical Failure',
    evaluatorLabel: 'Evaluator',
    noKpiDefined: 'No KPIs defined',
    criticalFailNote: 'Critical failure (if score < 60, the whole interaction fails)',    isOpenStatus: 'Open',
    isClosedStatus: 'Closed',
    isActiveAgent: 'Active',
    isInactiveAgent: 'Inactive',
    csvDownloadedToast: 'CSV file downloaded',
    yesText: 'Yes',
    noText: 'No',
        // Priority pill labels
    priorityHigh: 'High',
    priorityMedium: 'Medium',
    priorityLow: 'Low',
    priorityLabel: 'Priority',
    
    // Severity pill labels
    severityCritical: 'Critical',
    severityHigh: 'High',
    severityMedium: 'Medium',
    severityLow: 'Low',
    
    // Customer risk labels
    riskHigh: 'High Risk',
    riskHighSub: 'Immediate contact needed',
    riskMed: 'Medium Risk',
    riskMedSub: 'Follow-up required',
    riskLow: 'Low Risk',
    riskLowSub: 'Normal monitoring',
    totalCustomersLabel: 'Total Customers',
    withRiskScoreLabel: 'With Risk Score',
    noDataAvailable: 'No data available',
    moreLabel: 'more',
    
    // Chart labels
    avgQualityScore: 'Average Quality Score',
    healthyLabel: 'Healthy (≥80)',
    needImproveLabel: 'Needs Improvement (60-80)',
    criticalLabel: 'Critical (<60)',
    noRecordLabel: 'No records',
    customPageSize: 'Custom...',
    pageSizePlaceholder: 'count',
    paginationInfo: function(start, end, total, page, totalPages) { 
        return `Showing <b>${start}–${end}</b> of <b>${total}</b> records (page ${page} of ${totalPages})`; 
    },
    firstPage: '« First',
    prevPage: '‹ Prev',
    nextPage: 'Next ›',
    lastPage: 'Last »',
    pageSizeLabel: 'Items per page:',
    pageSizePrompt: 'Enter page size (1-1000):',
    allAgentsLabel: 'All Agents',
    autoScoreProgress: function(count) { return `Auto-scoring ${count} KPIs...`; },
    interactionDetailTitle: 'Interaction Details',
    transcriptLabel: 'Transcript:',
    criticalFailLabel: 'Critical Failure',
    evaluatorNote: 'Evaluator Notes (optional)',
    notesPlaceholder: 'Your additional notes',
    saveToDashboard: 'Save to Dashboard',
    cancelBtn: 'Cancel',
    saveBtn: 'Save',
    noActiveKpiError: 'No active KPIs',
    allScoredMsg: 'All interactions scored. Great job!',
    expertCustomerChannel: function(agent, customer, channel) { 
        return `Agent: ${agent || '-'} | Customer: ${customer || '-'} | Channel: ${channel}`; 
    },
    modalCloseBtn: 'Close',
    editUserLabel: 'Edit Password / Role',
    deleteBtn: 'Delete',
    cannotDeleteSelf: 'Cannot delete yourself',
    confirmDelete: function(username) { return `Delete user "${username}"?`; },
    userDeletedToast: 'User deleted',
    newUserTitle: 'Create New User',
    usernameLabel: 'Username (min 3 chars)',
    passwordLabel: 'Password (min 4 chars)',
    adminAccessLabel: 'System Admin Access',
    createBtn: 'Create',
    usernameRequired: 'Username is required',
    userCreatedToast: 'User created',
    editUserTitle: function(username) { return `Edit User: ${username}`; },
    newPasswordLabel: 'New Password (leave blank to keep)',
    updatedToast: 'User updated',
    onlyAdminAccessible: 'Only system administrators have access',
    // Product types
    productBank: 'Bank',
    productInsurance: 'Insurance',
    productInvestment: 'Investment',
    productLoan: 'Loan',
    
    // Buttons
    editBtn: 'Edit',
    deleteBtn: 'Delete',
    saveBtn: 'Save',
    cancelBtn: 'Cancel',
    closeBtn: 'Close',
    reportBtn: 'Report',
    
    // Messages
    emptyCustomerMsg: 'No customers found',
    customerDeletedToast: 'Deleted',
    confirmDelete: function(name) { return `Delete ${name}?`; },
    isActiveAgent: 'Active',
    isInactiveAgent: 'Inactive',
    
    // Risk levels
    riskHigh: 'High Risk',
    riskMed: 'Medium Risk',
    riskLow: 'Low Risk',
    riskHighSub: 'Immediate contact needed',
    riskMedSub: 'Follow-up required',
    riskLowSub: 'Normal monitoring',
    totalCustomersLabel: 'Total Customers',
    withRiskScoreLabel: 'With Risk Score',
    noDataAvailable: 'No data available',
    moreLabel: 'more',
    // More UI strings
    pageInfoLabel: function(s,e,t,p,tp) { return `نمایش <b>${s}–${e}</b> از <b>${t}</b> رکورد (صفحه ${p} از ${tp})`; },
    pageSizeLabel: 'تعداد در صفحه:',
    pageSizePrompt: 'تعداد در صفحه را وارد کنید (۱ تا ۱۰۰۰):',
    allAgentsOpt: 'همه کارشناسان',
    autoScoreProgress: function(n) { return `در حال اندازهگیری خودکار ${n} KPI...`; },
    interactionDetailTitle: 'جزئیات تعامل',
    transcriptLabel: 'متن مکالمه:',
    criticalFailLabel2: 'شکست بحرانی',
    evaluatorNote: 'یادداشت ارزیاب (اختیاری)',
    notesPlaceholder: 'نکات تکمیلی شما',
    saveToDashboardBtn: 'ذخیره در داشبورد',
    cancelBtn2: 'انصراف',
    noActiveKpi: 'هیچ KPI فعالی',
    kpiMsg: 'ابتدا KPI تعریف کنید یا پیشفرضها را بارگذاری کنید',
    expertCustomerChannel: function(a,c,ch) { return `کارشناس: ${a || '-'} | مشتری: ${c || '-'} | کانال: ${ch}`; },
    riskLabel: 'ریسک',
    riskFactorsLabel: 'دلایل ریسک:',
    suggestedActionLabel: 'اقدام پیشنهادی:',
    autoScoreBtn2: 'ارزیابی خودکار',
    activeLabel: 'فعال',
    inactiveLabel: 'غیرفعال',
    reportBtn2: 'گزارش',
    toggleToast: function(active) { return active ? 'فعال شد' : 'غیرفعال شد'; },
    agentNameLabel: 'نام و نام خانوادگی',
    agentDeptLabel: 'واحد',
    agentPosLabel: 'سمت',
    agentSavedToast: 'کارشناس ثبت شد',
    editCustLabel: 'ویرایش',
    deleteCustLabel: 'حذف',
    segmentLabel: 'سطح',
    segmentNormal: 'عادی',
    segmentImportant: 'مهم',
    notesFieldLabel: 'یادداشت',
    customerUpdatedToast: 'مشتری بهroزرسانی شد',
    newCustomerName: 'نام',
    newCustomerPhone: 'تلفن',
    newCustomerProduct: 'محصول',
    newCustomerSeg: 'بخش',
    normalOpt: 'عادی',
    vipOpt: 'VIP',
    corpOpt: 'شرکتی',
    customerCreatedToast: 'مشتری ثبت شد',
    // More UI strings
    pageInfoLabel: (s,e,t,p,tp) => `Showing <b>${s}–${e}</b> of <b>${t}</b> records (page ${p} of ${tp})`,
    pageSizeLabel: 'Items per page:',
    pageSizePrompt: 'Enter page size (1-1000):',
    allAgentsOpt: 'All Agents',
    autoScoreProgress: (n) => `Auto-scoring ${n} KPIs...`,
    interactionDetailTitle: 'Interaction Details',
    transcriptLabel: 'Transcript:',
    criticalFailLabel2: 'Critical Failure',
    evaluatorNote: 'Evaluator Notes (optional)',
    notesPlaceholder: 'Your additional notes',
    saveToDashboardBtn: 'Save to Dashboard',
    cancelBtn2: 'Cancel',
    noActiveKpi: 'No active KPIs',
    kpiMsg: 'Define KPIs first or load defaults',
    expertCustomerChannel: (a,c,ch) => `Agent: ${a || '-'} | Customer: ${c || '-'} | Channel: ${ch}`,
    riskLabel: 'Risk',
    riskFactorsLabel: 'Risk Factors:',
    suggestedActionLabel: 'Suggested Action:',
    autoScoreBtn2: 'Auto Score',
    activeLabel: 'Active',
    inactiveLabel: 'Inactive',
    reportBtn2: 'Report',
    toggleToast: (active) => active ? 'Activated' : 'Deactivated',
    agentNameLabel: 'Full Name',
    agentDeptLabel: 'Department',
    agentPosLabel: 'Position',
    agentSavedToast: 'Agent registered',
    editCustLabel: 'Edit',
    deleteCustLabel: 'Delete',
    segmentLabel: 'Level',
    segmentNormal: 'Normal',
    segmentImportant: 'Important',
    notesFieldLabel: 'Notes',
    customerUpdatedToast: 'Customer updated',
    newCustomerName: 'Name',
    newCustomerPhone: 'Phone',
    newCustomerProduct: 'Product',
    newCustomerSeg: 'Segment',
    normalOpt: 'Normal',
    vipOpt: 'VIP',
    corpOpt: 'Corporate',
    customerCreatedToast: 'Customer saved',
    // Score pills
    scoreNotEvaluated: 'Not Evaluated',
    emptyTable: 'No data available',

    // Buttons
    editBtn: 'Edit',
    deleteBtn: 'Delete',
    saveBtn: 'Save',
    cancelBtn: 'Cancel',
    closeBtn: 'Close',
    reportBtn: 'Report',
    
    // Messages
    emptyCustomerMsg: 'No customers found',
    customerDeletedToast: 'Deleted',
    confirmDelete: (name) => `Delete ${name}?`,
    isActiveAgent: 'Active',
    isInactiveAgent: 'Inactive',
    
    // Product types
    productBank: 'Bank',
    productInsurance: 'Insurance',
    productInvestment: 'Investment',
    productLoan: 'Loan',
    // Error messages
    invalidResponse: 'Invalid response',

    // RTL/LTR
    dir: 'ltr',
    lang: 'en',
  }
};

// Current language state
let currentLang = localStorage.getItem('crm_qi_lang') || 'fa';

/**
 * Get translated string
 * @param {string} key - Translation key
 * @param {...*} args - Arguments for function values
 * @returns {string} Translated text
 */
function t(key, ...args) {
  const dict = I18N[currentLang] || I18N.fa;
  let value = dict[key];
  if (typeof value === 'function') {
    value = value(...args);
  }
  return value || key;
}

/**
 * Apply language to DOM
 */
function applyLanguage(lang) {
  currentLang = lang;
  localStorage.setItem('crm_qi_lang', lang);

  const dict = I18N[lang] || I18N.fa;
  const html = document.documentElement;
  html.setAttribute('lang', dict.lang);
  html.setAttribute('dir', dict.dir);
  html.classList.toggle('rtl', dict.dir === 'rtl');
  html.classList.toggle('ltr', dict.dir === 'ltr');

  // Update page title
  document.title = dict.pageTitle;

  // Update nav items
  const navMap = {
    'dashboard': 'navDashboard',
    'interactions': 'navInteractions',
    'agents': 'navAgents',
    'customers': 'navCustomers',
    'risk': 'navRisk',
    'recommendations': 'navRecommendations',
    'issues': 'navIssues',
    'coaching': 'navCoaching',
    'calibration': 'navCalibration',
    'rubrics': 'navRubrics',
    'report': 'navReport',
    'users': 'navUsers',
    'audit': 'navAudit'
  };

  Object.entries(navMap).forEach(([tab, key]) => {
    const el = document.querySelector(`[data-tab="${tab}"]`);
    if (el) {
      const svg = el.querySelector('svg');
      if (svg) {
        el.innerHTML = '';
        el.appendChild(svg.cloneNode(true));
        el.appendChild(document.createTextNode(' ' + dict[key]));
      }
    }
  });

  // Update static text elements with data-i18n attribute
  document.querySelectorAll('[data-i18n]').forEach(el => {
    const key = el.getAttribute('data-i18n');
    if (dict[key]) {
      if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {
        el.placeholder = dict[key];
      } else {
        el.textContent = dict[key];
      }
    }
  });

  // Update search placeholder
  const searchInput = document.getElementById('fSearch');
  if (searchInput) {
    searchInput.placeholder = dict.searchPlaceholder;
  }

  // Update channel filter options
  const channelSelect = document.getElementById('fChannel');
  if (channelSelect) {
    const currentVal = channelSelect.value;
    channelSelect.innerHTML = `<option value="">${dict.allChannels}</option>`;
    dict.channelOptions.forEach(opt => {
      channelSelect.innerHTML += `<option value="${opt}">${opt}</option>`;
    });
    channelSelect.value = currentVal;
  }

  // Update status filter options
  const statusSelect = document.getElementById('iStatus');
  if (statusSelect) {
    const currentVal = statusSelect.value;
    statusSelect.innerHTML = `<option value="">${dict.allStatuses}</option>`;
    dict.statusOptions.forEach(opt => {
      statusSelect.innerHTML += `<option>${opt}</option>`;
    });
    statusSelect.value = currentVal;
  }

  // Update severity filter options
  const severitySelect = document.getElementById('iSeverity');
  if (severitySelect) {
    const currentVal = severitySelect.value;
    severitySelect.innerHTML = `<option value="">${dict.allSeverities}</option>`;
    dict.severityOptions.forEach(opt => {
      severitySelect.innerHTML += `<option>${opt}</option>`;
    });
    severitySelect.value = currentVal;
  }

  // Update coaching status filter
  const cStatusSelect = document.getElementById('cStatus');
  if (cStatusSelect) {
    const currentVal = cStatusSelect.value;
    cStatusSelect.innerHTML = `<option value="">${dict.allStatuses}</option>`;
    dict.coachingStatusOptions.forEach(opt => {
      cStatusSelect.innerHTML += `<option value="${opt}">${opt}</option>`;
    });
    cStatusSelect.value = currentVal;
  }

  // Update calibration status filter
  const calStatusSelect = document.getElementById('calStatus');
  if (calStatusSelect) {
    const currentVal = calStatusSelect.value;
    calStatusSelect.innerHTML = `<option value="">${dict.allStatuses}</option>`;
    dict.calibrationStatusOptions.forEach(opt => {
      calStatusSelect.innerHTML += `<option value="${opt}">${opt}</option>`;
    });
    calStatusSelect.value = currentVal;
  }

  // Update language switcher button text
  const langBtn = document.getElementById('langBtn');
  if (langBtn) {
    langBtn.textContent = lang === 'fa' ? 'EN' : 'فارسی';
  }

  // Update mobile nav items
  document.querySelectorAll('.mobile-nav-item').forEach(item => {
    const tab = item.dataset.tab;
    if (navMap[tab] && dict[navMap[tab]]) {
      const svg = item.querySelector('svg');
      item.innerHTML = '';
      item.appendChild(svg.cloneNode(true));
      item.appendChild(document.createTextNode(' ' + dict[navMap[tab]]));
    }
  });

  // Re-render current page to update dynamic content
  const activeTab = document.querySelector('.nav-item.active')?.dataset.tab;
  if (activeTab) {
    window.switchTab?.(activeTab);
  }
}

// Initialize language on load
document.addEventListener('DOMContentLoaded', () => {
  applyLanguage(currentLang);
});
